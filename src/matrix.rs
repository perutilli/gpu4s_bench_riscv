type Number = i32;

#[derive(Debug)]
pub struct Matrix<
    'a,
    const SIDE: usize,
    const SIZE: usize,
    const SECTION_SIZE: usize,
    const N_SECTIONS: usize,
> {
    data: [Number; SIZE],
    rows: usize,
    cols: usize,
    _phantomdata: &'a (), // letting 'a be a lifetime parameter
}

impl<
        'a,
        const SIDE: usize,
        const SIZE: usize,
        const SECTION_SIZE: usize,
        const N_SECTIONS: usize,
    > Matrix<'a, SIDE, SIZE, SECTION_SIZE, N_SECTIONS>
where
    [Option<MatrixSection<'a, SECTION_SIZE, SIDE, SIZE, N_SECTIONS>>; N_SECTIONS]: Default, // this means that the maximum number of sections is 32 => i.e. N_HARTS <= 32
{
    pub const fn zeroes() -> Self {
        Matrix {
            data: [0; SIZE],
            rows: SIDE,
            cols: SIDE,
            _phantomdata: &(),
        }
    }

    pub const fn from_slice(data: [Number; SIZE]) -> Self {
        Matrix {
            data,
            rows: SIDE,
            cols: SIDE,
            _phantomdata: &(),
        }
    }

    pub fn sections_mut(
        &mut self,
    ) -> [Option<MatrixSection<'_, SECTION_SIZE, SIDE, SIZE, N_SECTIONS>>; N_SECTIONS] {
        let mut sections: [Option<MatrixSection<'_, SECTION_SIZE, SIDE, SIZE, N_SECTIONS>>;
            N_SECTIONS] = Default::default();
        self.data
            .chunks_exact_mut(SECTION_SIZE)
            .enumerate()
            .for_each(|(i, section)| {
                sections[i] = Some(MatrixSection::new(
                    section.try_into().unwrap(),
                    self.rows,
                    self.cols,
                    i,
                ))
            });
        sections
    }
}

#[derive(Debug)]
pub struct MatrixSection<
    'a,
    const SIZE: usize,
    const MATRIX_SIDE: usize,
    const MATRIX_SIZE: usize,
    const N_SECTIONS: usize,
> {
    section_data: &'a mut [Number; SIZE],
    rows: usize,
    cols: usize,
    section_number: usize,
}

impl<
        'a,
        const SIZE: usize,
        const MATRIX_SIDE: usize,
        const MATRIX_SIZE: usize,
        const N_SECTIONS: usize,
    > MatrixSection<'a, SIZE, MATRIX_SIDE, MATRIX_SIZE, N_SECTIONS>
{
    pub fn new(
        section_data: &'a mut [Number; SIZE],
        rows: usize,
        cols: usize,
        section_number: usize,
    ) -> Self {
        MatrixSection {
            section_data,
            rows,
            cols,
            section_number,
        }
    }

    pub fn multiply(
        &mut self,
        a: &Matrix<MATRIX_SIDE, MATRIX_SIZE, MATRIX_SIZE, 1>,
        b: &Matrix<MATRIX_SIDE, MATRIX_SIZE, MATRIX_SIZE, 1>,
    ) {
        self.section_data
            .iter_mut()
            .enumerate()
            .for_each(|(i, elem)| {
                let row = (self.section_number * self.rows + i) / self.cols;
                let col = i % self.cols;
                for k in 0..self.cols {
                    *elem += a.data[row * a.cols + k] * b.data[k * b.cols + col];
                }
            });
    }
}

impl<
        'a,
        const MATRIX_SIDE: usize,
        const MATRIX_SIZE: usize,
        const SECTION_SIZE: usize,
        const N_SECTIONS: usize,
        const KERNEL_SIDE: usize,
        const KERNEL_SIZE: usize,
    > Convolution<MATRIX_SIDE, MATRIX_SIZE, SECTION_SIZE, N_SECTIONS, KERNEL_SIDE, KERNEL_SIZE>
    for MatrixSection<'a, SECTION_SIZE, MATRIX_SIDE, MATRIX_SIZE, N_SECTIONS>
{
    fn convolute(
        &mut self,
        a: &Matrix<MATRIX_SIDE, MATRIX_SIZE, SECTION_SIZE, N_SECTIONS>,
        kernel: &Matrix<KERNEL_SIDE, KERNEL_SIZE, 0, 0>,
    ) {
        let kernel_y_radius = (kernel.rows - 1) / 2;
        let kernel_x_radius = (kernel.cols - 1) / 2;
        self.section_data
            .iter_mut()
            .enumerate()
            .for_each(|(i, elem)| {
                let row = (self.section_number * self.rows + i) / self.cols;
                let col = i % self.cols;
                for k in 0..kernel.rows {
                    for l in 0..kernel.cols {
                        let y = (row + k) as isize - kernel_y_radius as isize;
                        let x = (col + l) as isize - kernel_x_radius as isize;
                        if (y >= 0 && y < self.rows as isize) && (x >= 0 && x < self.cols as isize)
                        {
                            *elem += a.data[y as usize * self.cols + x as usize]
                                * kernel.data[k * kernel.cols + l];
                        }
                    }
                }
            });
    }
}

pub trait Convolution<
    const MATRIX_SIDE: usize,
    const MATRIX_SIZE: usize,
    const SECTION_SIZE: usize,
    const N_SECTIONS: usize,
    const KERNEL_SIDE: usize,
    const KERNEL_SIZE: usize,
>
{
    fn convolute(
        &mut self,
        a: &Matrix<MATRIX_SIDE, MATRIX_SIZE, SECTION_SIZE, N_SECTIONS>,
        kernel: &Matrix<KERNEL_SIDE, KERNEL_SIZE, 0, 0>,
    );
}
