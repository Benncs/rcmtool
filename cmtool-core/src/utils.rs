use crate::coordinates::*;
use crate::ensight_gold::types::{ElementsType, VolumeElementTypes};
use crate::grid::MeshType;

#[inline(always)]
pub fn linear_index_2d_matrix_row_major(i_coord: usize, i_axis: usize, n_col: usize) -> usize {
    n_col * i_coord + i_axis
}
#[inline(always)]
pub fn linear_index_coordinates_matrix(i_coord: usize, i_axis: usize) -> usize {
    linear_index_2d_matrix_row_major(i_coord, i_axis, NUMBER_OF_AXIS)
}

pub type AxisPoints = [usize; NUMBER_OF_AXIS];

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

// fn tetra_area(
//     CartesianCoordinates(a): CartesianCoordinates,
//     CartesianCoordinates(b): CartesianCoordinates,
//     CartesianCoordinates(c): CartesianCoordinates,
//     CartesianCoordinates(d): CartesianCoordinates,
//     value_on_ax: f64,
//     axe_index: usize,
// ) -> f64 {
//     let mut intersections = Vec::new();

//     // Add vertices that lie exactly on the plane
//     for &pt in &[a, b, c, d] {
//         if (pt[axe_index] - value_on_ax).abs() < 1e-10 {
//             intersections.push(pt);
//         }
//     }

//     // All tetrahedron edges
//     let edges = [(a, b), (a, c), (a, d), (b, c), (b, d), (c, d)];

//     for (start, end) in edges.iter() {
//         if let Some(intersect) = intersect_line_plane(*start, *end, axe_index, value_on_ax) {
//             intersections.push(intersect);
//         }
//     }

//     if intersections.len() < 3 {
//         return 0.0; // Not enough points to form a polygon
//     }

//     // Project and sort the intersection polygon
//     let projected = project_and_sort_polygon(&intersections, axe_index);

//     calculate_polygon_area(&projected)
// }

fn tetra_area(
    a: CartesianCoordinates,
    b: CartesianCoordinates,
    c: CartesianCoordinates,
    d: CartesianCoordinates,
    value_on_ax: f64,
    axe_index: usize,
) -> f64 {
    let mut intersections = Vec::new();

    // Add vertices that lie exactly on the plane
    for &pt in &[a, b, c, d] {
        if (pt.0[axe_index] - value_on_ax).abs() < 1e-10 {
            intersections.push(pt.0);
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
    CartesianCoordinates(a): CartesianCoordinates,
    CartesianCoordinates(b): CartesianCoordinates,
    axe_index: usize,
    value_on_ax: f64,
) -> Option<Coords3> {
    let ab = b.sub(&a);
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

fn normalize(v: Coords3) -> Coords3 {
    let norm = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    [v[0] / norm, v[1] / norm, v[2] / norm]
}

fn orthonormal_basis(normal: Coords3) -> (Coords3, Coords3) {
    let mut u = if normal[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let d = normal.dot(&u);
    u = [
        u[0] - d * normal[0],
        u[1] - d * normal[1],
        u[2] - d * normal[2],
    ];
    u = normalize(u);
    let v = normal.cross(&u);
    (u, v)
}

fn tetra_area_cylindrical(
    a: CartesianCoordinates,
    b: CartesianCoordinates,
    c: CartesianCoordinates,
    d: CartesianCoordinates,
    value_on_ax: f64,
    axis_index: usize,
) -> f64 {
    let points_cyl = [a, b, c, d].map(CylindricalCoordinates::from);

    let mut intersections = Vec::new();

    for pt in &points_cyl {
        if (pt.0[axis_index] - value_on_ax).abs() < 1e-10 {
            intersections.push(CartesianCoordinates::from(pt).0);
        }
    }

    let edges = [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)];
    for &(i0, i1) in &edges {
        if let Some(intersect) = intersect_line_plane_cylindrical(
            CartesianCoordinates([a.0, b.0, c.0, d.0][i0]),
            CartesianCoordinates([a.0, b.0, c.0, d.0][i1]),
            axis_index,
            value_on_ax,
        ) {
            intersections.push(intersect);
        }
    }

    if intersections.len() < 3 {
        return 0.0;
    }

    let projected: Vec<[f64; 2]> = intersections
        .iter()
        .map(|p| {
            match axis_index {
                0 => {
                    // r=const
                    let cyl = CylindricalCoordinates::from(CartesianCoordinates(*p));
                    [cyl.0[1], cyl.0[2]]
                }
                1 => {
                    // θ=const
                    let cyl = CylindricalCoordinates::from(CartesianCoordinates(*p));
                    [cyl.0[0], cyl.0[2]]
                }
                2 => {
                    // z=const
                    let cyl = CylindricalCoordinates::from(CartesianCoordinates(*p));
                    [cyl.0[0], cyl.0[1]]
                }
                _ => panic!("Invalid axis_index"),
            }
        })
        .collect();

    let projected_sorted = sort_polygon_ccw(&projected);
    calculate_polygon_area(&projected_sorted)
}

fn intersect_line_plane_cylindrical(
    CartesianCoordinates(a): CartesianCoordinates,
    CartesianCoordinates(b): CartesianCoordinates,
    axis_index: usize,
    value_on_ax: f64,
) -> Option<Coords3> {
    let ab = b.sub(&a);

    match axis_index {
        0 => {
            // r = const
            // (x(t)^2 + y(t)^2 = r^2)
            let A = ab[0] * ab[0] + ab[1] * ab[1];
            if A.abs() < 1e-14 {
                return None;
            }
            let B = 2.0 * (a[0] * ab[0] + a[1] * ab[1]);
            let C = a[0] * a[0] + a[1] * a[1] - value_on_ax * value_on_ax;

            let discr = B * B - 4.0 * A * C;
            if discr < 0.0 {
                return None;
            }

            let sqrt_discr = discr.sqrt();
            let t_candidates = [(-B - sqrt_discr) / (2.0 * A), (-B + sqrt_discr) / (2.0 * A)];
            t_candidates
                .iter()
                .copied()
                .filter(|&t| (0.0..=1.0).contains(&t))
                .min_by(|x, y| x.partial_cmp(y).unwrap())
                .map(|t| [a[0] + t * ab[0], a[1] + t * ab[1], a[2] + t * ab[2]])
        }
        1 => {
            // θ = const
            let normal = [value_on_ax.sin(), -value_on_ax.cos(), 0.0];
            let denom = normal[0] * ab[0] + normal[1] * ab[1];
            if denom.abs() < 1e-14 {
                return None;
            }
            let t = -(normal[0] * a[0] + normal[1] * a[1]) / denom;
            if (0.0..=1.0).contains(&t) {
                Some([a[0] + t * ab[0], a[1] + t * ab[1], a[2] + t * ab[2]])
            } else {
                None
            }
        }
        2 => {
            // z = const
            intersect_line_plane(
                CartesianCoordinates(a),
                CartesianCoordinates(b),
                axis_index,
                value_on_ax,
            )
        }
        _ => None,
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

    let projected: Vec<[f64; 2]> = points.iter().map(|p| [p[i1], p[i2]]).collect();

    sort_polygon_ccw(&projected)
}

fn sort_polygon_ccw(points: &[[f64; 2]]) -> Vec<[f64; 2]> {
    let centroid = {
        let (mut sx, mut sy) = (0.0, 0.0);
        for p in points {
            sx += p[0];
            sy += p[1];
        }
        [sx / points.len() as f64, sy / points.len() as f64]
    };

    let mut sorted = points.to_vec();
    sorted.sort_by(|a, b| {
        let angle_a = (a[1] - centroid[1]).atan2(a[0] - centroid[0]);
        let angle_b = (b[1] - centroid[1]).atan2(b[0] - centroid[0]);
        angle_a.partial_cmp(&angle_b).unwrap()
    });
    sorted
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
fn tetra_volume(
    CartesianCoordinates(a): CartesianCoordinates,
    CartesianCoordinates(b): CartesianCoordinates,
    CartesianCoordinates(c): CartesianCoordinates,
    CartesianCoordinates(d): CartesianCoordinates,
) -> f64 {
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

pub fn compute_intersection_area(
    local_vertices: &[CartesianCoordinates],
    elem_type: VolumeElementTypes,
    value_on_ax: f64,
    axis_index: usize,
    mestype: MeshType,
) -> Option<f64> {
    let callback = match mestype {
        MeshType::Cylindrical => tetra_area_cylindrical,
        _ => tetra_area,
    };

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
            callback(a, b, c, d, value_on_ax, axis_index)
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
    fn test_intersection_area_r_plane() {
        use std::f64::consts::PI;

        let a = CylindricalCoordinates([1.0, 0.0, 0.0]);
        let b = CylindricalCoordinates([1.0, PI / 2.0, 0.0]);
        let c = CylindricalCoordinates([1.0, 0.0, 1.0]);
        let d = CylindricalCoordinates([0.5, 0.0, 0.0]);

        let ac = [c.0[0] - a.0[0], c.0[1] - a.0[1], c.0[2] - a.0[2]];
        let ab = [b.0[0] - a.0[0], b.0[1] - a.0[1], b.0[2] - a.0[2]];
        let cross_prod = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];

        let expected_area = 0.5
            * (cross_prod[0] * cross_prod[0]
                + cross_prod[1] * cross_prod[1]
                + cross_prod[2] * cross_prod[2])
                .sqrt();

        let a = CartesianCoordinates::from(a);
        let b = CartesianCoordinates::from(b);
        let c = CartesianCoordinates::from(c);
        let d = CartesianCoordinates::from(d);

        let vertices = vec![a, b, c, d];

        let area = compute_intersection_area(
            &vertices,
            VolumeElementTypes::Tetra4,
            1.0,
            0,
            MeshType::Cylindrical,
        )
        .unwrap();

        assert!(
            (area - expected_area).abs() < 1e-12,
            "{} != {}",
            area,
            expected_area
        );
    }

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
        let area = tetra_area(
            CartesianCoordinates(a),
            CartesianCoordinates(b),
            CartesianCoordinates(c),
            CartesianCoordinates(d),
            value_on_ax,
            axe_index,
        );

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
}
