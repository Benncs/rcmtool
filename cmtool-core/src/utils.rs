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

    fn cylindrical_to_cartesian(&self) -> Self;
    fn cartesian_to_cylindrical(&self) -> Self;
}

pub fn vector_cartesian_to_cylindrical(
    [vx, vy, vz]: &Coords3,
    [_r, theta, _z]: Coords3,
) -> Coords3 {
    let cos_theta = theta.cos();
    let sin_theta = theta.sin();

    [
        vx * cos_theta + vy * sin_theta,
        -vx * sin_theta + vy * cos_theta,
        *vz,
    ]
}

impl Coords3Ext for Coords3 {
    fn sub(&self, other: &Self) -> Self {
        [self[0] - other[0], self[1] - other[1], self[2] - other[2]]
    }
    fn cylindrical_to_cartesian(&self) -> Self {
        [self[0] * self[1].cos(), self[0] * self[1].sin(), self[2]]
    }

    fn cartesian_to_cylindrical(&self) -> Self {
        // let r = f64::sqrt(self[0]*self[0]+self[1]*self[1]);
        // let theta = if(self[0]>=0. && r>0.)
        // {
        //     f64::acos(self[0]/r)
        // }
        // else if(self[0]<0. && r>0.)
        // {
        //     -f64::acos(self[0]/r)
        // }
        // else if( r==0.)
        // {
        //     0.
        // }
        // else {
        //     panic!("Unable to compute cartesian")
        // };

        // [r,theta,self[2]]

        let r = f64::sqrt(self[0] * self[0] + self[1] * self[1]);
        let theta = self[1].atan2(self[0]); // handles all cases
        [r, theta, self[2]]
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

fn tetra_area(
    a: Coords3,
    b: Coords3,
    c: Coords3,
    d: Coords3,
    value_on_ax: f64,
    axe_index: usize,
) -> f64 {
    let mut intersections = Vec::new();

    // Add vertices that lie exactly on the plane
    for &pt in &[a, b, c, d] {
        if (pt[axe_index] - value_on_ax).abs() < 1e-10 {
            intersections.push(pt);
        }
    }

    // All tetrahedron edges
    let edges = [(a, b), (a, c), (a, d), (b, c), (b, d), (c, d)];

    for (start, end) in edges.iter() {
        if let Some(intersect) = intersect_line_plane(*start, *end, axe_index, value_on_ax) {
            intersections.push(intersect);
        }
    }

    if intersections.len() < 3 {
        return 0.0; // Not enough points to form a polygon
    }

    // Project and sort the intersection polygon
    let projected = project_and_sort_polygon(&intersections, axe_index);

    calculate_polygon_area(&projected)
}

fn intersect_line_plane(
    a: Coords3,
    b: Coords3,
    axe_index: usize,
    value_on_ax: f64,
) -> Option<Coords3> {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let denom = ab[axe_index];

    if denom.abs() < 1e-10 {
        return None; // Parallel to plane
    }

    let t = (value_on_ax - a[axe_index]) / denom;

    if (0.0..=1.0).contains(&t) {
        Some([a[0] + t * ab[0], a[1] + t * ab[1], a[2] + t * ab[2]])
    } else {
        None
    }
}

/// Project 3D points onto the slicing plane and sort them counter-clockwise
fn project_and_sort_polygon(points: &[Coords3], axe_index: usize) -> Vec<[f64; 2]> {
    // Determine which 2D plane to project onto
    let (i1, i2) = match axe_index {
        0 => (1, 2), // yz-plane
        1 => (0, 2), // xz-plane
        2 => (0, 1), // xy-plane
        _ => panic!("Invalid axis index"),
    };

    let mut projected: Vec<[f64; 2]> = points.iter().map(|p| [p[i1], p[i2]]).collect();

    // Sort the projected points counter-clockwise
    let centroid = {
        let (mut sx, mut sy) = (0.0, 0.0);
        for p in &projected {
            sx += p[0];
            sy += p[1];
        }
        [sx / projected.len() as f64, sy / projected.len() as f64]
    };

    projected.sort_by(|a, b| {
        let angle_a = (a[1] - centroid[1]).atan2(a[0] - centroid[0]);
        let angle_b = (b[1] - centroid[1]).atan2(b[0] - centroid[0]);
        angle_a.partial_cmp(&angle_b).unwrap()
    });

    projected
}

fn calculate_polygon_area(points: &[[f64; 2]]) -> f64 {
    if points.len() < 3 {
        return 0.0;
    }

    let mut area = 0.0;
    let n = points.len();
    for i in 0..n {
        let j = (i + 1) % n;
        area += points[i][0] * points[j][1];
        area -= points[j][0] * points[i][1];
    }
    area.abs() / 2.0
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

//Local vertices in xyz coordinates is mandatory
pub fn compute_intersection_area(
    local_vertices: &[Coords3],
    elem_type: VolumeElementTypes,
    value_on_ax: f64,
    axe_index: usize,
) -> Option<f64> {
    let tetra_indices = elem_type.tetra_subdivisions();
    if local_vertices.len() != ElementsType::VolumeElementType(elem_type).node_count() as usize {
        return None;
    }
    let area = tetra_indices
        .iter()
        .map(|&[i0, i1, i2, i3]| {
            let a = local_vertices[i0];
            let b = local_vertices[i1];
            let c = local_vertices[i2];
            let d = local_vertices[i3];
            tetra_area(a, b, c, d, value_on_ax, axe_index)
        })
        .sum();

    Some(area)
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
    fn test_tetrahedron_intersection_area() {
        // Define a simple tetrahedron with one vertex on each axis
        let a = [0.0, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [0.0, 1.0, 0.0];
        let d = [0.0, 0.0, 1.0];

        // Define the slicing plane z = 0.5
        let value_on_ax = 0.5;
        let axe_index = 2; // z-axis

        // Calculate the intersection area
        let area = tetra_area(a, b, c, d, value_on_ax, axe_index);

        // Manually compute the triangle formed by slicing at z = 0.5
        // The intersection points are:
        // - a to d: [0, 0, 0] to [0, 0, 1] → [0, 0, 0.5]
        // - b to d: [1, 0, 0] to [0, 0, 1] → [0.5, 0, 0.5]
        // - c to d: [0, 1, 0] to [0, 0, 1] → [0, 0.5, 0.5]
        //
        // Projecting these onto the XY plane:
        // [0.0, 0.0], [0.5, 0.0], [0.0, 0.5]
        //
        // Shoelace formula:
        // Area = 0.5 * |(x1*y2 + x2*y3 + x3*y1) - (x2*y1 + x3*y2 + x1*y3)|
        // Area = 0.5 * |(0*0 + 0.5*0.5 + 0*0) - (0.5*0 + 0*0.5 + 0*0.5)| = 0.125

        let expected_area = 0.125;

        assert!(
            (area - expected_area).abs() < 1e-10,
            "Expected area {}, got {}",
            expected_area,
            area
        );
    }

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
