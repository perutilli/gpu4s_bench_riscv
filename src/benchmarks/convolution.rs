#[cfg(feature = "sequential")]
mod benchmark {
    const SIDE: usize = 4;
    const SIZE: usize = SIDE * SIDE;

    const KERNEL_SIDE: usize = 3;
    const KERNEL_SIZE: usize = KERNEL_SIDE * KERNEL_SIDE;

    use crate::matrix::{Convolution, Matrix};
    use crate::{print, println};

    const A: Matrix<SIDE, SIZE, SIZE, 1> =
        Matrix::from_slice([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
    const KERNEL: Matrix<KERNEL_SIDE, KERNEL_SIZE, 0, 0> =
        Matrix::from_slice([0, 1, 2, 3, 4, 5, 6, 7, 8]);
    #[no_mangle]
    extern "C" fn main(hart_id: usize) {
        assert_eq!(hart_id, 0);

        let mut C: Matrix<SIDE, SIZE, SIZE, 1> = Matrix::zeroes();

        let t = crate::time();
        for section in C.sections_mut() {
            section
                .expect("We expect this to be set")
                .convolute(&A, &KERNEL);
        }

        println!("Time: {:?}", crate::time() - t);
        println!("Result: {:?}", C);
    }
}

#[cfg(any(
    feature = "parallel",
    not(any(feature = "sequential", feature = "parallel"))
))]
mod benchmark {
    use crate::N_HARTS;
    const SIDE: usize = 4;
    const SIZE: usize = SIDE * SIDE;
    const SECTION_SIZE: usize = SIZE / N_HARTS;
    const N_SECTIONS: usize = N_HARTS;

    const KERNEL_SIDE: usize = 3;
    const KERNEL_SIZE: usize = KERNEL_SIDE * KERNEL_SIDE;

    use crate::matrix::{Convolution, Matrix};
    use crate::shared_matrix::SharedMatrix;
    use crate::{print, println};

    const A: Matrix<SIDE, SIZE, SECTION_SIZE, N_SECTIONS> =
        Matrix::from_slice([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
    const KERNEL: Matrix<KERNEL_SIDE, KERNEL_SIZE, 0, 0> =
        Matrix::from_slice([0, 1, 2, 3, 4, 5, 6, 7, 8]);

    static C: SharedMatrix<SIDE, SIZE, SECTION_SIZE, N_SECTIONS> =
        SharedMatrix::new(Matrix::zeroes());

    #[no_mangle]
    extern "C" fn main(hart_id: usize) {
        if hart_id == 0 {
            println!("Convolution");
        }

        C.initialize();
        let t = crate::time(); // start timer after initialization, we will use the hart 0 timer

        // C.convolute(&A, &KERNEL, hart_id);
        C.compute(
            |section| {
                section.convolute(&A, &KERNEL);
            },
            hart_id,
        );

        if hart_id == 0 {
            println!("Time: {:?}", crate::time() - t);
            println!("Result: {}", C);
        }
    }
}
