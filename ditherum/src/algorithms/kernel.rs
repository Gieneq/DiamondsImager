/// Represents a mutable 2x2 kernel over a matrix.
/// 
/// This struct provides mutable references to four adjacent elements in a 2x2 region.
pub struct MutKernel2x2<'a, T> {
    pub tl: &'a mut T,  // Top-left element
    pub tr: &'a mut T,  // Top-right element
    pub bl: &'a mut T,  // Bottom-left element
    pub br: &'a mut T,  // Bottom-right element
}

/// Applies a 2x2 kernel-based processing function to a mutable matrix.
/// 
/// This function iterates over the matrix and calls the provided function on each 2x2 submatrix.
/// If the kernel extends beyond the matrix bounds, default values are used.
/// 
/// # Parameters
/// - `matrix`: A mutable reference to a 2D vector.
/// - `processing`: A function that takes a `MutKernel2x2<T>` and modifies the matrix accordingly.
/// 
/// # Panics
/// Panics if the matrix has fewer than two rows or columns.
pub fn apply_2x2_kernel_processing<T, P>(matrix: &mut [Vec<T>], mut processing: P)
where 
    T: Default,
    P: FnMut(MutKernel2x2<T>)
{
    let height = matrix.len();
    assert!(height > 1);
    let width = matrix[0].len();
    assert!(width > 1);

    let mut dummy_tr = T::default();
    let mut dummy_bl = T::default();
    let mut dummy_br = T::default();

    for y in 0..height {
        let row_is_last = y == (height - 1);

        for x in 0..width {
            let column_is_last = x == (width - 1);

            let tl = (&mut matrix[y][x]) as *mut T;

            let tr = if !column_is_last { &mut matrix[y][x + 1] } else { &mut dummy_tr } as *mut T;
            
            let bl = if !row_is_last { &mut matrix[y + 1][x] } else { &mut dummy_bl } as *mut T;
            
            let br = if !row_is_last && !column_is_last { &mut matrix[y + 1][x + 1] } else { &mut dummy_br } as *mut T;

            unsafe {
                let kernel = MutKernel2x2 {
                    tl: &mut *tl,
                    tr: &mut *tr,
                    bl: &mut *bl,
                    br: &mut *br
                };
                processing(kernel);
            }
        }
    }
}

#[test]
fn test_unsafe_kernel_processing_simple() {
    let mut data = vec![vec![0u8; 2]; 2];
    apply_2x2_kernel_processing(&mut data, |kernel| {
        *kernel.tl += 1;
        *kernel.tr += 1;
        *kernel.bl += 1;
        *kernel.br += 1;
    });
    let processed_data = data;
    let expected_data = vec![vec![1, 2], vec![2, 4]];
    assert_eq!(processed_data, expected_data);
}