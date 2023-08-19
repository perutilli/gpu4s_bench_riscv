use crate::matrix::{Matrix, MatrixSection};
use crate::N_HARTS;
use core::cell::UnsafeCell;
use core::fmt::Display;
use core::sync::atomic::{AtomicBool, AtomicUsize};

const ATOMIC_TRUE: AtomicBool = AtomicBool::new(true);

#[derive(Debug)]
pub struct SharedMatrix<
    'a,
    const SIDE: usize,
    const SIZE: usize,
    const SECTION_SIZE: usize,
    const N_SECTIONS: usize,
> {
    matrix: UnsafeCell<Matrix<'a, SIDE, SIZE, SECTION_SIZE, N_SECTIONS>>,
    sections: UnsafeCell<
        Option<[Option<MatrixSection<'a, SECTION_SIZE, SIDE, SIZE, N_SECTIONS>>; N_SECTIONS]>,
    >,
    initializing: AtomicBool,
    initialized: AtomicBool,
    section_available: [AtomicBool; N_SECTIONS],
    computation_completed: AtomicUsize,
}

impl<
        'a,
        const SIDE: usize,
        const SIZE: usize,
        const SECTION_SIZE: usize,
        const N_SECTIONS: usize,
    > SharedMatrix<'a, SIDE, SIZE, SECTION_SIZE, N_SECTIONS>
where
    [Option<MatrixSection<'a, SECTION_SIZE, SIDE, SIZE, N_SECTIONS>>; N_SECTIONS]: Default, // this means that the maximum number of sections is 32 => i.e. N_HARTS <= 32
{
    /// This needs to be called once in a static context
    /// There is no need to enforce this since it is the only way to initialize the matrix
    /// and have access to it from different harts
    pub const fn new(init_value: Matrix<'a, SIDE, SIZE, SECTION_SIZE, N_SECTIONS>) -> Self {
        SharedMatrix {
            matrix: UnsafeCell::new(init_value),
            sections: UnsafeCell::new(None),
            initializing: AtomicBool::new(false),
            initialized: AtomicBool::new(false),
            section_available: [ATOMIC_TRUE; N_SECTIONS],
            computation_completed: AtomicUsize::new(0),
        }
    }

    /// Initializes the matrix (for now only sets the sections)
    /// Call this function from at least one hart
    /// The hart that will deal with the initialization will be decided by a race
    pub fn initialize(&self) {
        // note that 2 flags are necessary, one to keep track if the initialization has started
        // and one to keep track if the initialization has completed
        if self
            .initializing
            .compare_exchange(
                false,
                true,
                core::sync::atomic::Ordering::SeqCst,
                core::sync::atomic::Ordering::SeqCst,
            )
            .is_err()
        {
            // some other thread has gotten here first
            // they will be in charge of the initialization
            return;
        }
        unsafe {
            assert!((*self.sections.get()).is_none(), "API internal error");
            (*self.sections.get()) = Some((*self.matrix.get()).sections_mut());
        }
        self.initialized
            .store(true, core::sync::atomic::Ordering::SeqCst);
    }

    /// Gets an exclusive reference to a section of the matrix once they have been set
    /// TODO: now we wait for all, we could spin on the specific one
    fn get_section(
        &self,
        section_idx: usize,
    ) -> MatrixSection<'a, SECTION_SIZE, SIDE, SIZE, N_SECTIONS> {
        // spin until the matrix is initialized
        while !self.initialized.load(core::sync::atomic::Ordering::SeqCst) {}
        unsafe {
            match (*self.sections.get()).as_mut() {
                None => unreachable!("This cannot be none if section_set is true, unless the code paniced, in which case the program should have aborted"),
                Some(sections) => {
                    if self.section_available[section_idx]
                        .compare_exchange(
                            true,
                            false,
                            core::sync::atomic::Ordering::SeqCst,
                            core::sync::atomic::Ordering::SeqCst,
                        )
                        .is_err()
                    {
                        panic!("Another thread already owns this section");
                    }
                    sections[section_idx].take().expect("By checking section_available, this should be unreachable")
                }
            }
        }
    }

    fn notify_completed(
        &self,
        section: MatrixSection<'a, SECTION_SIZE, SIDE, SIZE, N_SECTIONS>,
        section_idx: usize,
    ) {
        self.computation_completed
            .fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        unsafe {
            (*self.sections.get())
                .as_mut()
                .expect("The computation has started, so the sections cannot be none")
                [section_idx] = Some(section);
        }
        // TODO: uncomment the following line once we have a mechanism to start a new computation
        // self.section_available[section_idx].store(true, core::sync::atomic::Ordering::SeqCst);
    }

    /*
    pub fn multiply(
        &'a self,
        a: &Matrix<'a, SIDE, SIZE, SECTION_SIZE, N_SECTIONS>,
        b: &Matrix<'a, SIDE, SIZE, SECTION_SIZE, N_SECTIONS>,
        section_idx: usize,
    ) {
        let mut section = self.get_section(section_idx);
        section.multiply(a, b);
        self.notify_completed(section, section_idx);
    }
     */

    pub fn compute(
        &'a self,
        compute_fn: impl FnOnce(&mut MatrixSection<'a, SECTION_SIZE, SIDE, SIZE, N_SECTIONS>),
        section_idx: usize,
    ) {
        let mut section = self.get_section(section_idx);
        compute_fn(&mut section);
        self.notify_completed(section, section_idx);
    }
    /*
    pub fn convolute(
        &'a self,
        a: &Matrix<'a, SIDE, SIZE, SECTION_SIZE, N_SECTIONS>,
        kernel: &Matrix<'a, SIDE, SIZE, SECTION_SIZE, N_SECTIONS>,
        section_idx: usize,
    ) {
        let mut section = self.get_section(section_idx);
        crate::const_generics_matrix::Convolution::convolute(&mut section, a, kernel);
        self.notify_completed(section, section_idx);
    }

    /// This spins until the matrix is available (i.e. all computations are completed)
    /// If a thread does not notify that it has completed its computation, this will spin forever
    pub fn wait_new_calculation(&self) {
        // spin until the computation is completed
        while self
            .computation_completed
            .load(core::sync::atomic::Ordering::SeqCst)
            != N_HARTS
        {}
        self.computation_completed
            .store(0, core::sync::atomic::Ordering::SeqCst);
    }
     */
}

impl<
        'a,
        const SIDE: usize,
        const SIZE: usize,
        const SECTION_SIZE: usize,
        const N_SECTIONS: usize,
    > Display for SharedMatrix<'a, SIDE, SIZE, SECTION_SIZE, N_SECTIONS>
{
    /// This spins until the matrix is available (i.e. all computations are completed)
    /// If a thread does not notify that it has completed its computation, this will spin forever
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        while self
            .computation_completed
            .load(core::sync::atomic::Ordering::SeqCst)
            != N_HARTS
        {}
        unsafe { write!(f, "{:?}", (*self.matrix.get())) }
    }
}

unsafe impl<
        'a,
        const SIDE: usize,
        const SIZE: usize,
        const SECTION_SIZE: usize,
        const N_SECTIONS: usize,
    > Sync for SharedMatrix<'a, SIDE, SIZE, SECTION_SIZE, N_SECTIONS>
{
}
