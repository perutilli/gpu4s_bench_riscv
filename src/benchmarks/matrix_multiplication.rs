mod benchmark {
    use crate::N_HARTS;
    const SIDE: usize = 4;
    const SIZE: usize = SIDE * SIDE;
    const SECTION_SIZE: usize = SIZE / N_HARTS;
    const N_SECTIONS: usize = N_HARTS;

    use crate::matrix::Matrix;
    use crate::shared_matrix::SharedMatrix;
    use crate::{print, println};

    const A: Matrix<SIDE, SIZE, SECTION_SIZE, N_SECTIONS> =
        Matrix::from_slice([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
    const B: Matrix<SIDE, SIZE, SECTION_SIZE, N_SECTIONS> =
        Matrix::from_slice([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);

    static C: SharedMatrix<SIDE, SIZE, SECTION_SIZE, N_SECTIONS> =
        SharedMatrix::new(Matrix::zeroes());

    #[no_mangle]
    extern "C" fn main(hart_id: usize) {
        if hart_id == 0 {
            println!("Matrix multiplication");
        }

        C.initialize();

        let t = crate::time(); // start timer after initialization, we will use the hart 0 timer
        C.compute(
            |section| {
                section.multiply(&A, &B);
            },
            hart_id,
        );

        // C.multiply(&A, &B, hart_id);

        if hart_id == 0 {
            println!("Time: {:?}", crate::time() - t);
            println!("Result: {}", C);
        }
    }
}
