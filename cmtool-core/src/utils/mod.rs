// SPDX-License-Identifier: GPL-3.0-or-later

use crate::coordinates::*;
use crate::ensight_gold::types::{ElementsType, VolumeElementTypes};
mod area;
pub use area::*;

#[inline(always)]
pub fn linear_index_2d_matrix_row_major(i_coord: usize, i_axis: usize, n_col: usize) -> usize {
    n_col * i_coord + i_axis
}
#[inline(always)]
pub fn linear_index_coordinates_matrix(i_coord: usize, i_axis: usize) -> usize {
    linear_index_2d_matrix_row_major(i_coord, i_axis, NUMBER_OF_AXIS)
}

pub type AxisPoints = [usize; NUMBER_OF_AXIS];

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
fn tetra_volume(
    a: CartesianCoordinates,
    b: CartesianCoordinates,
    c: CartesianCoordinates,
    d: CartesianCoordinates,
) -> f64 {
    // let ab = b.sub(&a);
    // let ac = c.sub(&a);
    // let ad = d.sub(&a);
    let ac = CartesianVec3::from_point(c, a);
    let ab = CartesianVec3::from_point(b, a);
    let ad = CartesianVec3::from_point(d, a);

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
            Self::Tetra4 | Self::GTetra4 => &[[0, 1, 2, 3]],
            Self::Pyramid5 | Self::GPyramid5 => &[[0, 1, 3, 4], [1, 2, 3, 4]],
            Self::Penta6 | Self::GPenta6 => &[[0, 1, 2, 3], [1, 2, 3, 4], [2, 3, 4, 5]],
            Self::Hexa8 | Self::GHexa8 => &[
                [0, 1, 3, 4],
                [1, 3, 4, 5],
                [3, 4, 5, 7],
                [2, 3, 1, 6],
                [3, 1, 6, 7],
                [1, 6, 7, 5],
            ],
            Self::Tetra10 | Self::GTetra10 => &[[0, 1, 2, 3]],
            Self::Pyramid13 | Self::GPyramid13 => &[[0, 1, 3, 4], [1, 2, 3, 4]],
            Self::Penta15 | Self::GPenta15 => &[[0, 1, 2, 3], [1, 2, 3, 4], [2, 3, 4, 5]],
            Self::Hexa20 | Self::GHexa20 => &[
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

pub fn compute_centroid<'a, I>(vertices: I) -> CartesianCoordinates
where
    I: Iterator<Item = &'a [f64; 3]>,
{
    let mut coords: Coords3 = Default::default();
    let mut count = 0;

    // for vertex in vertices {
    //     coords
    //         .iter_mut()
    //         .zip(vertex.iter())
    //         .for_each(|(c, v)| *c += *v);
    //     count += 1;
    // }
    // if count > 0 {
    //     coords.iter_mut().for_each(|c| *c /= count as f64);
    // }

    //SIMD friendly version ? Even if it's not the case, loop unroling here is still very readable
    for vertex in vertices {
        coords[0] += vertex[0];
        coords[1] += vertex[1];
        coords[2] += vertex[2];
        count += 1;
    }

    //Same here loop unrolling may improve SIMD and still elegant
    if count > 0 {
        let inv_count = 1.0 / count as f64;
        coords[0] *= inv_count;
        coords[1] *= inv_count;
        coords[2] *= inv_count;
    }

    CartesianCoordinates(coords)
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
pub fn compute_volume(
    local_vertices: &[CartesianCoordinates],
    elem_type: VolumeElementTypes,
) -> Option<f64> {
    let tetra_indices = elem_type.tetra_subdivisions();

    if local_vertices.len() != ElementsType::VolumeElementType(elem_type).node_count() as usize {
        return None;
    }

    let vol = tetra_indices
        .iter()
        .map(|&[i0, i1, i2, i3]| {
            let a = local_vertices[i0];
            let b = local_vertices[i1];
            let c = local_vertices[i2];
            let d = local_vertices[i3];
            tetra_volume(a, b, c, d)
        })
        .sum();

    Some(vol)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_tetrahedron_volume() {
        let a = CartesianCoordinates([0.0, 0.0, 0.0]);
        let b = CartesianCoordinates([1.0, 0.0, 0.0]);
        let c = CartesianCoordinates([0.0, 1.0, 0.0]);
        let d = CartesianCoordinates([0.0, 0.0, 1.0]);

        let vol = tetra_volume(a, b, c, d);
        assert!((vol - 1.0 / 6.0).abs() < 1e-10);
    }

    #[test]
    fn hexa8_unit_cube_volume() {
        let vertices: Vec<CartesianCoordinates> = vec![
            [0.0, 0.0, 0.0], // 0
            [1.0, 0.0, 0.0], // 1
            [1.0, 1.0, 0.0], // 2
            [0.0, 1.0, 0.0], // 3
            [0.0, 0.0, 1.0], // 4
            [1.0, 0.0, 1.0], // 5
            [1.0, 1.0, 1.0], // 6
            [0.0, 1.0, 1.0], // 7
        ]
        .into_iter()
        .map(CartesianCoordinates)
        .collect();

        let volume = compute_volume(&vertices, VolumeElementTypes::Hexa8);
        assert!(volume.is_some());
        let volume = volume.unwrap();
        assert!(
            (volume - 1.0).abs() < 1e-10,
            "Expected volume ≈ 1.0, got {}",
            volume
        );
    }

    #[test]
    fn translated_hexa8_cube_volume() {
        let size = 2.0;
        let offset = 3.0;

        let vertices: Vec<CartesianCoordinates> = vec![
            [offset, offset, offset],
            [offset + size, offset, offset],
            [offset + size, offset + size, offset],
            [offset, offset + size, offset],
            [offset, offset, offset + size],
            [offset + size, offset, offset + size],
            [offset + size, offset + size, offset + size],
            [offset, offset + size, offset + size],
        ]
        .into_iter()
        .map(CartesianCoordinates)
        .collect();

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

    #[test]
    fn test_centroid_triangle_2d() {
        let v1 = &[0.0, 0.0, 0.0];
        let v2 = &[1.0, 0.0, 0.0];
        let v3 = &[0.0, 1.0, 0.0];

        let centroid = compute_centroid([v1, v2, v3].iter().copied());

        assert_eq!(centroid.0, [1.0 / 3.0, 1.0 / 3.0, 0.0]);
    }

    #[test]
    fn test_centroid_hexa() {
        //Vertices are chosen to be -1 0 or 1 to have origin as centroid
        let cube_vertices = [
            &[-1.0, -1.0, -1.0],
            &[1.0, -1.0, -1.0],
            &[1.0, 1.0, -1.0],
            &[-1.0, 1.0, -1.0],
            &[-1.0, -1.0, 1.0],
            &[1.0, -1.0, 1.0],
            &[1.0, 1.0, 1.0],
            &[-1.0, 1.0, 1.0],
        ];

        let centroid = compute_centroid(cube_vertices.iter().copied());

        assert!(
            centroid.0.iter().all(|c| c.abs() < 1e-12),
            "Centroid is not at origin: got {:?}",
            centroid.0
        );
    }

    #[test]
    fn test_centroid_tetraheadron() {

        //for tetra: Centroid=1/4​(A+B+C+D)

        let tetrahedron_vertices = [
            &[0.0, 0.0, 0.0],
            &[2.0, 0.0, 0.0],
            &[0.0, 2.0, 0.0],
            &[2.0, 2.0, 4.0],
        ];

        let centroid = compute_centroid(tetrahedron_vertices.iter().copied());

        let expect_centroid = [1., 1., 1.];
        centroid
        .0
        .iter()
        .zip(expect_centroid)
        .for_each(|(c, e)| assert!((c - e).abs() < 1e-12, "expected {e}, got {c}"));
    }
}
