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

// pub fn compute_intersection_area(
//     local_vertices: &[CartesianCoordinates],
//     elem_type: VolumeElementTypes,
//     value_on_ax: f64,
//     axis_index: usize,
//     mestype: MeshType,
// ) -> Option<f64> {
//     let callback = match mestype {
//         MeshType::Cylindrical => tetra_area_cylindrical,
//         _ => tetra_area,
//     };

//     let tetra_indices = elem_type.tetra_subdivisions();
//     if local_vertices.len() != ElementsType::VolumeElementType(elem_type).node_count() as usize {
//         return None;
//     }
//     let area = tetra_indices
//         .iter()
//         .map(|&[i0, i1, i2, i3]| {
//             let a = local_vertices[i0];
//             let b = local_vertices[i1];
//             let c = local_vertices[i2];
//             let d = local_vertices[i3];
//             callback(a, b, c, d, value_on_ax, axis_index)
//         })
//         .sum();

//     Some(area)
// }

fn project_points_to_plane_2d(points: &Vec<[f64; 3]>, normal: &Coords3) -> Vec<[f64; 2]> {
    let n = normal.normalized();
    let arbitrary = if n[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let u = n.cross(&arbitrary).normalized();
    let v = n.cross(&u);

    points.iter().map(|p| [p.dot(&u), p.dot(&v)]).collect()
}

fn polygon_area_2d(points: &Vec<[f64; 2]>) -> f64 {
    let n = points.len();
    let mut area = 0.0;
    for i in 0..n {
        let (x0, y0) = (points[i][0], points[i][1]);
        let (x1, y1) = (points[(i + 1) % n][0], points[(i + 1) % n][1]);
        area += x0 * y1 - x1 * y0;
    }
    area.abs() * 0.5
}

fn tetra_area(vertices: [CartesianCoordinates; 4], plane: &Plane) -> f64 {
    let value = 0.;
    let mut intersection_points = vec![];
    if let Plane::Vector { normal, point:CartesianCoordinates(point) } = plane {
        let d = -normal.dot(point); // plane offset
        let distances: Vec<f64> = vertices
            .iter()
            .map(|CartesianCoordinates(v)| normal.dot(v) + d)
            .collect();

        for i in 0..4 {
            for j in (i + 1)..4 {
                let d1 = distances[i];
                let d2 = distances[j];

                if d1 * d2 < 0.0 {
                    let t = d1 / (d1 - d2);
                    let p1 = &vertices[i].0;
                    let p2 = &vertices[j].0;
                    let intersection = [
                        p1[0] + t * (p2[0] - p1[0]),
                        p1[1] + t * (p2[1] - p1[1]),
                        p1[2] + t * (p2[2] - p1[2]),
                    ];
                    intersection_points.push(intersection);
                }
            }
        }

        if intersection_points.len() < 3 {
            return 0.0; // No intersection area
        }

        // Step 3: Project points to 2D plane
        let projected = project_points_to_plane_2d(&intersection_points, normal);

        // Step 4: Sort points counterclockwise
        let sorted = sort_polygon_ccw(&projected);

        // Step 5: Compute area using shoelace formula
        return polygon_area_2d(&sorted);
    }

    value
}

pub fn _compute_intersection_area(
    local_vertices: &[CartesianCoordinates],
    elem_type: VolumeElementTypes,
    plane: &Plane,
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
            1.
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
        
        

        let plane = &Plane::Vector { normal: [0.,0.,0.], point: CartesianCoordinates([0.,0.,0.]) };

        let vertices = [a, b, c, d];
        
        let area = tetra_area(vertices, plane);

        // let area = compute_intersection_area(
        //     &vertices,
        //     VolumeElementTypes::Tetra4,
        //     1.0,
        //     0,
        //     MeshType::Cylindrical,
        // )
        // .unwrap();

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

    
        let plane = Plane::Vector {
            normal: [0., 0., 1.],
            point: CartesianCoordinates([0., 0., 0.5]),
        };

        // Calculate the intersection area
        // let area = tetra_area(
        //     CartesianCoordinates(a),
        //     CartesianCoordinates(b),
        //     CartesianCoordinates(c),
        //     CartesianCoordinates(d),
        //     value_on_ax,
        //     axe_index,
        // );

        let area = tetra_area(
            [
                CartesianCoordinates(a),
                CartesianCoordinates(b),
                CartesianCoordinates(c),
                CartesianCoordinates(d),
            ],
            &plane,
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
