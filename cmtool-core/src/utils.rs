use crate::ensight_gold::types::{ElementsType, VolumeElementTypes};

const NUMBER_OF_AXIS: usize = 3;

#[inline(always)]
pub fn linear_index_2d_matrix_row_major(i_coord: usize, i_axis: usize, n_col: usize) -> usize {
    n_col * i_coord + i_axis
}
#[inline(always)]
pub fn linear_index_coordinates_matrix(i_coord: usize, i_axis: usize) -> usize {
    linear_index_2d_matrix_row_major(i_coord, i_axis, NUMBER_OF_AXIS)
}


pub type Coords3 = [f64; NUMBER_OF_AXIS];
pub type AxisPoints = [usize; NUMBER_OF_AXIS];



pub trait Coords3Ext {
    fn sub(&self, other: &Self) -> Self;
    fn cross(&self, other: &Self) -> Self;
    fn dot(&self, other: &Self) -> f64;
}

impl Coords3Ext for Coords3 {
    fn sub(&self, other: &Self) -> Self {
        [self[0] - other[0], self[1] - other[1], self[2] - other[2]]
    }

    fn cross(&self, other: &Self) -> Self {
        [
            self[1] * other[2] - self[2] * other[1],
            self[2] * other[0] - self[0] * other[2],
            self[0] * other[1] - self[1] * other[0],
        ]
    }

    fn dot(&self, other: &Self) -> f64 {
        self[0] * other[0] + self[1] * other[1] + self[2] * other[2]
    }
}


/// Computes the signed volume of a tetrahedron defined by four 3D points.
///
/// # Arguments
///
/// * `a`, `b`, `c`, `d` - The vertices of the tetrahedron (3D coordinates).
///
/// # Returns
///
/// The absolute value of the mixed product divided by 6, which gives the
/// volume of the tetrahedron.
///
fn tetra_volume(a: Coords3, b: Coords3, c: Coords3, d: Coords3) -> f64 {
    let ab = b.sub(&a);
    let ac = c.sub(&a);
    let ad = d.sub(&a);
    ab.dot(&ac.cross(&ad)).abs() / 6.0
}

impl VolumeElementTypes {
    /// Returns the list of tetrahedral subdivisions for this volume element type.
    ///
    /// Each 4-index array represents one tetrahedron by specifying vertex indices
    /// within the original element. These are used to compute the total volume
    /// of the element as a sum of sub-tetrahedron volumes.
    ///
    /// The subdivisions follow the same decomposition logic as the original
    /// `c_numbering_tetra` table from the C++ implementation.
    ///
    /// # Returns
    ///
    /// A reference to a static array of vertex index groups (`[usize; 4]`),
    /// where each group defines one tetrahedron.
    fn tetra_subdivisions(&self) -> &'static [[usize; 4]] {
        match self {
            Self::Tetra4|Self::GTetra4 => &[
                [0, 1, 2, 3],
            ],
            Self::Pyramid5|Self::GPyramid5 => &[
                [0, 1, 3, 4],
                [1, 2, 3, 4],
            ],
            Self::Penta6| Self::GPenta6 => &[
                [0, 1, 2, 3],
                [1, 2, 3, 4],
                [2, 3, 4, 5],
            ],
            Self::Hexa8|Self::GHexa8 => &[
                [0, 1, 3, 4],
                [1, 3, 4, 5],
                [3, 4, 5, 7],
                [2, 3, 1, 6],
                [3, 1, 6, 7],
                [1, 6, 7, 5],
            ],
            Self::Tetra10|Self::GTetra10 => &[
                [0, 1, 2, 3],
            ],
            Self::Pyramid13|Self::GPyramid13 => &[
                [0, 1, 3, 4],
                [1, 2, 3, 4],
            ],
            Self::Penta15|Self::GPenta15 => &[
                [0, 1, 2, 3],
                [1, 2, 3, 4],
                [2, 3, 4, 5],
            ],
            Self::Hexa20|Self::GHexa20 => &[
                [0, 1, 3, 4],
                [1, 3, 4, 5],
                [3, 4, 5, 7],
                [2, 3, 1, 6],
                [3, 1, 6, 7],
                [1, 6, 7, 5],
            ],
            
        }
    }
}

/// Computes the total volume of a given volume element by summing the
/// volumes of its tetrahedral subdivisions.
///
/// # Arguments
///
/// * `local_vertices` - A slice of the 3D coordinates of the element's vertices.
/// * `elem_type` - The type of the volume element (e.g., Hexa8, Penta6).
///
/// # Returns
///
/// An `Option<f64>` containing the total volume if the input is valid.
/// Returns `None` if the number of vertices does not match the element type.
///
pub fn compute_volume(local_vertices: &[Coords3], elem_type: VolumeElementTypes) -> Option<f64> {
    let tetra_indices = elem_type.tetra_subdivisions();

    if local_vertices.len()!=ElementsType::VolumeElementType(elem_type).node_count() as usize
    {
        return None;
    }


    let vol = tetra_indices.iter().map(|&[i0, i1, i2, i3]| {
        let a = local_vertices[i0];
        let b = local_vertices[i1];
        let c = local_vertices[i2];
        let d = local_vertices[i3];
        tetra_volume(a, b, c, d)
    }).sum();

    Some(vol)
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_tetrahedron_volume() {
        let a = [0.0, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [0.0, 1.0, 0.0];
        let d = [0.0, 0.0, 1.0];

        let vol = tetra_volume(a, b, c, d);
        assert!((vol - 1.0 / 6.0).abs() < 1e-10);
    }

    #[test]
    fn hexa8_unit_cube_volume() {
        let vertices: Vec<[f64; 3]> = vec![
            [0.0, 0.0, 0.0], // 0
            [1.0, 0.0, 0.0], // 1
            [1.0, 1.0, 0.0], // 2
            [0.0, 1.0, 0.0], // 3
            [0.0, 0.0, 1.0], // 4
            [1.0, 0.0, 1.0], // 5
            [1.0, 1.0, 1.0], // 6
            [0.0, 1.0, 1.0], // 7
        ];

        let volume = compute_volume(&vertices, VolumeElementTypes::Hexa8);
        assert!(volume.is_some());
        let volume = volume.unwrap();
        assert!((volume - 1.0).abs() < 1e-10, "Expected volume ≈ 1.0, got {}", volume);
    }

    #[test]
    fn translated_hexa8_cube_volume() {
        let size = 2.0;
        let offset = 3.0;

        let vertices: Vec<[f64; 3]> = vec![
            [offset, offset, offset],
            [offset + size, offset, offset],
            [offset + size, offset + size, offset],
            [offset, offset + size, offset],
            [offset, offset, offset + size],
            [offset + size, offset, offset + size],
            [offset + size, offset + size, offset + size],
            [offset, offset + size, offset + size],
        ];

        let volume = compute_volume(&vertices, VolumeElementTypes::Hexa8);
        assert!(volume.is_some());
        let volume = volume.unwrap();
        let expected = size.powi(3);
        assert!(
            (volume - expected).abs() < 1e-10,
            "Expected volume ≈ {}, got {}",
            expected,
            volume
        );
    }
}