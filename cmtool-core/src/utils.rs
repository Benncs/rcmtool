const number_of_axis: usize = 3;

#[inline(always)]
pub fn linear_index_2d_matrix_row_major(i_coord: usize, i_axis: usize, n_col: usize) -> usize {
    n_col * i_coord + i_axis
}
#[inline(always)]
pub fn linear_index_coordinates_matrix(i_coord: usize, i_axis: usize) -> usize {
    linear_index_2d_matrix_row_major(i_coord, i_axis, number_of_axis)
}
