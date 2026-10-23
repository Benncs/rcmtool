// SPDX-License-Identifier: GPL-3.0-or-later

use crate::coordinates::*;
mod area;
pub use area::*;
mod polygon;
mod volumes;

pub use volumes::{compute_centroid, compute_volume};

#[inline(always)]
pub fn linear_index_2d_matrix_row_major(i_coord: usize, i_axis: usize, n_col: usize) -> usize {
    n_col * i_coord + i_axis
}
#[inline(always)]
pub fn linear_index_coordinates_matrix(i_coord: usize, i_axis: usize) -> usize {
    linear_index_2d_matrix_row_major(i_coord, i_axis, NUMBER_OF_AXIS)
}

pub type AxisPoints = [usize; NUMBER_OF_AXIS];
