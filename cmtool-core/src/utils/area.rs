// SPDX-License-Identifier: GPL-3.0-or-later

use super::polygon::{HalfSpace, Polygon};
use crate::{
    coordinates::*,
    ensight_gold::types::{ElementsType, VolumeElementTypes},
};

// fn sort_polygon_ccw(points: &[[f64; 2]]) -> Vec<[f64; 2]> {
//     let centroid = {
//         let (mut sx, mut sy) = (0.0, 0.0);
//         for p in points {
//             sx += p[0];
//             sy += p[1];
//         }
//         [sx / points.len() as f64, sy / points.len() as f64]
//     };

//     let mut sorted = points.to_vec();
//     sorted.sort_by(|a, b| {
//         let angle_a = (a[1] - centroid[1]).atan2(a[0] - centroid[0]);
//         let angle_b = (b[1] - centroid[1]).atan2(b[0] - centroid[0]);
//         angle_a.partial_cmp(&angle_b).unwrap()
//     });
//     sorted
// }

fn project_points_to_plane_2d(points: &[[f64; 3]], normal: &CartesianVec3) -> Vec<[f64; 2]> {
    let n = normal.normalized();
    let arbitrary = if n.0[0].abs() < n.0[2].abs() {
        CartesianVec3([1.0, 0.0, 0.0])
    } else {
        CartesianVec3([0.0, 0.0, 1.0])
    };
    // let u = n.cross(&arbitrary).normalized();
    // let v = u.cross(&n).normalized();
    let v = n.cross(&arbitrary).normalized();
    let u = v.cross(&n).normalized();

    points
        .iter()
        .map(|p| CartesianVec3::from_point_origin(CartesianCoordinates(*p)))
        .map(|p| [p.dot(&u), p.dot(&v)])
        .collect()
}

// fn sort_points_ccw_3d(points: &[[f64; 3]], normal: &CartesianVec3) -> Vec<[f64; 3]> {
//     let projected = project_points_to_plane_2d(points, normal);
//     let sorted_2d = sort_polygon_ccw(&projected);

//     let mut sorted_3d = Vec::with_capacity(points.len());

//     for p2d in &sorted_2d {
//         let idx = projected
//             .iter()
//             .enumerate()
//             .min_by(|(_, a), (_, b)| {
//                 let da = (a[0] - p2d[0]).hypot(a[1] - p2d[1]);
//                 let db = (b[0] - p2d[0]).hypot(b[1] - p2d[1]);
//                 da.partial_cmp(&db).unwrap()
//             })
//             .map(|(i, _)| i)
//             .unwrap();
//         sorted_3d.push(points[idx]);
//     }

//     sorted_3d
// }
//
fn sort_points_ccw_3d(points: &[[f64; 3]], normal: &CartesianVec3) -> Vec<[f64; 3]> {
    let projected = project_points_to_plane_2d(points, normal);
    let mut indices: Vec<usize> = (0..points.len()).collect();
    let centroid = {
        let (mut sx, mut sy) = (0.0, 0.0);
        for p in &projected {
            sx += p[0];
            sy += p[1];
        }
        [sx / projected.len() as f64, sy / projected.len() as f64]
    };
    indices.sort_by(|&a, &b| {
        let angle_a = (projected[a][1] - centroid[1]).atan2(projected[a][0] - centroid[0]);
        let angle_b = (projected[b][1] - centroid[1]).atan2(projected[b][0] - centroid[0]);
        angle_a.partial_cmp(&angle_b).unwrap()
    });

    indices.iter().map(|&i| points[i]).collect()
}

///Index of the radial axis, the only face of a cylindrical compartment that is not a plane
const RADIAL_AXIS: usize = 0;

///Index of the axial axis, whose face is flat but bounded by two arcs
const AXIAL_AXIS: usize = 2;

///A radial face is a cylindrical patch, every other face of the compartment is flat
pub fn is_curved_face(face: &BoundedPlane) -> bool {
    face.axis == RADIAL_AXIS
}

///Plane to cut one element with, when the face it crosses is curved.
///
///`get_interface_plane` gives the plane tangent at the middle of the compartment, which drifts
///away from the patch as theta moves:
///
///```text
///      tangent at theta_c
///     ------+------            an element sitting here never reaches the tangent plane,
///      __--- ---__             it contributes no area at all
///    _-     |     -_  <- patch
///   /       |       \
///          axis
///```
///
///Taking the tangent at the angular position of the element keeps the error down to the curvature
///over one element instead of over one compartment.
pub fn tangent_plane_at(
    patch: &BoundedPlane,
    CartesianCoordinates(element_centroid): CartesianCoordinates,
) -> BoundedPlane {
    let CartesianCoordinates(patch_origin) = patch.origin;
    let radius = patch_origin[0].hypot(patch_origin[1]);
    let theta = element_centroid[1].atan2(element_centroid[0]);

    BoundedPlane {
        normal: CartesianVec3([theta.cos(), theta.sin(), 0.]),
        origin: CartesianCoordinates([radius * theta.cos(), radius * theta.sin(), patch_origin[2]]),
        extent_u: patch.extent_u,
        extent_v: patch.extent_v,
        axis: patch.axis,
    }
}

///Half spaces bounding the face of a compartment, when all of them are planes.
///So a radial and a theta face are exactly clippable by half spaces, an axial face is not: its
///r bounds are cylinders, and that case is left to the caller.
fn planar_bounds(plane: &BoundedPlane) -> Vec<HalfSpace> {
    let z_bounds = |z0: f64, z1: f64| {
        [
            HalfSpace {
                normal: CartesianVec3([0., 0., 1.]),
                offset: z0,
            },
            HalfSpace {
                normal: CartesianVec3([0., 0., -1.]),
                offset: -z1,
            },
        ]
    };

    //Two half spaces can only describe a sector narrower than a half turn
    let theta_bounds = |theta0: f64, theta1: f64| {
        if theta1 - theta0 >= std::f64::consts::PI {
            return None;
        }
        Some([
            HalfSpace {
                normal: CartesianVec3([-theta0.sin(), theta0.cos(), 0.]),
                offset: 0.,
            },
            HalfSpace {
                normal: CartesianVec3([theta1.sin(), -theta1.cos(), 0.]),
                offset: 0.,
            },
        ])
    };

    let mut bounds = Vec::with_capacity(4);
    match plane.axis {
        //Radial face: theta and z bounds
        0 => {
            bounds.extend(
                theta_bounds(plane.extent_u[0], plane.extent_u[1])
                    .into_iter()
                    .flatten(),
            );
            bounds.extend(z_bounds(plane.extent_v[0], plane.extent_v[1]));
        }
        //Theta face: radial and z bounds, the radial direction is the one of the face itself
        1 => {
            let CartesianCoordinates(origin) = plane.origin;
            let theta = origin[1].atan2(origin[0]);
            let radial = CartesianVec3([theta.cos(), theta.sin(), 0.]);

            bounds.push(HalfSpace {
                normal: radial,
                offset: plane.extent_u[0],
            });
            bounds.push(HalfSpace {
                normal: CartesianVec3([-radial.0[0], -radial.0[1], 0.]),
                offset: -plane.extent_u[1],
            });
            bounds.extend(z_bounds(plane.extent_v[0], plane.extent_v[1]));
        }
        //Axial face: only the theta bounds are planes, the radial ones are arcs
        _ => bounds.extend(
            theta_bounds(plane.extent_v[0], plane.extent_v[1])
                .into_iter()
                .flatten(),
        ),
    }

    bounds
}

///Signed area of the intersection between the triangle (origin, a, b) and the disk of radius
///`radius` centred on the origin.
///
///Summed over the edges of a polygon it gives the area of that polygon clipped to the disk, the
///same way the shoelace formula sums signed triangles. A piece of edge running outside the disk
///contributes its circular sector instead of its triangle:
///
///```text
///        b
///       /                 outside -> sector of the circle
///   ---+---___            inside  -> plain triangle
///  /  p2       \
/// |     \       |
/// |      p1     |
///  \      \    /
///   ---    a---
///```
fn triangle_disk_area(a: [f64; 2], b: [f64; 2], radius: f64) -> f64 {
    let cross = |u: [f64; 2], v: [f64; 2]| u[0] * v[1] - u[1] * v[0];
    let dot = |u: [f64; 2], v: [f64; 2]| u[0] * v[0] + u[1] * v[1];
    //Area swept on the circle between two directions
    let sector = |u: [f64; 2], v: [f64; 2]| 0.5 * radius * radius * cross(u, v).atan2(dot(u, v));

    let edge = [b[0] - a[0], b[1] - a[1]];
    let quadratic_a = dot(edge, edge);
    if quadratic_a < f64::EPSILON {
        return 0.;
    }
    let quadratic_b = 2. * dot(a, edge);
    let quadratic_c = dot(a, a) - radius * radius;
    let discriminant = quadratic_b * quadratic_b - 4. * quadratic_a * quadratic_c;

    if discriminant <= 0. {
        return sector(a, b);
    }

    let root = discriminant.sqrt();
    let entering = (-quadratic_b - root) / (2. * quadratic_a);
    let leaving = (-quadratic_b + root) / (2. * quadratic_a);

    //The edge crosses the circle outside of its own span
    if entering > 1. || leaving < 0. {
        return sector(a, b);
    }

    let at = |t: f64| [a[0] + t * edge[0], a[1] + t * edge[1]];
    let entering_point = at(entering.clamp(0., 1.));
    let leaving_point = at(leaving.clamp(0., 1.));

    sector(a, entering_point)
        + 0.5 * cross(entering_point, leaving_point)
        + sector(leaving_point, b)
}

///Area of a polygon of the plane z = constant kept inside the annulus of the axial face
fn polygon_annulus_area(polygon: &[Coords3], radii: [f64; 2]) -> f64 {
    //A radius is never negative, whatever a caller passes as bounds
    let radii = [radii[0].max(0.), radii[1].max(0.)];
    let disk_area = |radius: f64| {
        (0..polygon.len())
            .map(|i| {
                let current = polygon[i];
                let next = polygon[(i + 1) % polygon.len()];
                triangle_disk_area([current[0], current[1]], [next[0], next[1]], radius)
            })
            .sum::<f64>()
    };

    (disk_area(radii[1]) - disk_area(radii[0])).abs()
}

fn polygon_area_3d(points: &[Coords3], normal: &CartesianVec3) -> f64 {
    let n = normal.normalized();

    let mut area_vec = CartesianVec3([0.0, 0.0, 0.0]);
    let n_pts = points.len();

    for i in 0..n_pts {
        let p1 = CartesianVec3(points[i]);
        let p2 = CartesianVec3(points[(i + 1) % n_pts]);

        let cross = p1.cross(&p2);

        area_vec = area_vec.add(&cross);
    }
    0.5 * (area_vec.dot(&n)).abs()
}

fn tetra_area(vertices: [CartesianCoordinates; 4], plane: &BoundedPlane) -> f64 {
    const REL_TOL_DISTANCE: f64 = 1e-6;
    const EPSILON: f64 = 1e-12; //f64::EPSILON
    let mut intersection_points = vec![];

    let BoundedPlane {
        normal,
        origin: point,
        ..
    } = plane;

    let d = -normal.dot(&CartesianVec3::from_point_origin(*point)); // plane offset
    let distances: Vec<f64> = vertices
        .iter()
        .map(|coords| CartesianVec3::from_point_origin(*coords))
        .map(|v| normal.dot(&v) + d)
        .collect();

    let mut points_on_plane = vec![];
    let edge_len = (0..4)
        .flat_map(|i| (i + 1..4).map(move |j| (i, j)))
        .map(|(i, j)| {
            let e = [
                vertices[i].0[0] - vertices[j].0[0],
                vertices[i].0[1] - vertices[j].0[1],
                vertices[i].0[2] - vertices[j].0[2],
            ];
            (e[0] * e[0] + e[1] * e[1] + e[2] * e[2]).sqrt()
        })
        .fold(0.0_f64, f64::max);
    if edge_len < EPSILON {
        return 0.0;
    }
    let tol = REL_TOL_DISTANCE * edge_len;

    for (i, dist) in distances.iter().enumerate() {
        if dist.abs() < tol {
            points_on_plane.push(vertices[i].0);
        }
    }

    for i in 0..4 {
        for j in (i + 1)..4 {
            let d1 = distances[i];
            let d2 = distances[j];
            if d1.abs() < tol || d2.abs() < tol {
                continue;
            }
            if d1 * d2 < 0.0 {
                let t = d1.abs() / (d1.abs() + d2.abs());
                let p1 = &vertices[i].0;
                let p2 = &vertices[j].0;
                let intersection = [
                    p1[0] + t * (p2[0] - p1[0]),
                    p1[1] + t * (p2[1] - p1[1]),
                    p1[2] + t * (p2[2] - p1[2]),
                ];
                intersection_points.push(intersection);
            }
            // if d1 * d2 < 0.0 {
            //     let t = d1.abs() / (d1.abs() + d2.abs());
            //     let p1 = &vertices[i].0;
            //     let p2 = &vertices[j].0;
            //     let r1 = (p1[0].powi(2) + p1[1].powi(2)).sqrt();
            //     let r2 = (p2[0].powi(2) + p2[1].powi(2)).sqrt();
            //     let theta1 = p1[1].atan2(p1[0]);
            //     let theta2 = p2[1].atan2(p2[0]);
            //     let r_int = r1 + t * (r2 - r1);
            //     let theta_int = theta1 + t * (theta2 - theta1);
            //     intersection_points.push([
            //         r_int * theta_int.cos(),
            //         r_int * theta_int.sin(),
            //         p1[2] + t * (p2[2] - p1[2]),
            //     ]);
            // }
        }
    }

    for p in &points_on_plane {
        let already_present = intersection_points.iter().any(|q| {
            let dx = q[0] - p[0];
            let dy = q[1] - p[1];
            let dz = q[2] - p[2];
            (dx * dx + dy * dy + dz * dz).sqrt() < tol
        });
        if !already_present {
            intersection_points.push(*p);
        }
    }
    // intersection_points.extend(points_on_plane.iter().cloned());

    if intersection_points.len() < 3 {
        return 0.0;
    }

    let sorted = sort_points_ccw_3d(&intersection_points, normal);

    //Clip against the bounds of the face, an element straddling them contributes its share
    let clipped = planar_bounds(plane)
        .iter()
        .fold(Polygon::from_slice(&sorted), |polygon, half_space| {
            polygon.clip(half_space)
        });

    if clipped.as_slice().len() < 3 {
        return 0.0;
    }

    //An axial face is bounded in r by two arcs, which no half space can describe
    if plane.axis == AXIAL_AXIS {
        return polygon_annulus_area(clipped.as_slice(), plane.extent_u);
    }

    polygon_area_3d(clipped.as_slice(), normal)
}

pub fn compute_intersection_area(
    local_vertices: &[CartesianCoordinates],
    elem_type: VolumeElementTypes,
    plane: &BoundedPlane,
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
            tetra_area([a, b, c, d], plane)
        })
        .sum();

    Some(area)
}

#[cfg(test)]
mod test {
    use super::*;
    fn make_bounded_plane(
        normal: CartesianVec3,
        origin: CartesianCoordinates,
        axis: usize,
    ) -> BoundedPlane {
        // let (u, v) = orthonormal_basis(&normal);

        let extent = [-10.0, 10.0];

        BoundedPlane {
            normal,
            origin,
            extent_u: extent,
            extent_v: extent,
            axis,
        }
    }

    #[test]
    fn test_intersection_area_r_plane() {
        use std::f64::consts::PI;

        let a = CartesianCoordinates::from(CylindricalCoordinates([1.0, 0.0, 0.0]));
        let b = CartesianCoordinates::from(CylindricalCoordinates([1.0, PI / 2.0, 0.0]));
        let c = CartesianCoordinates::from(CylindricalCoordinates([1.0, 0.0, 1.0]));
        let d = CartesianCoordinates::from(CylindricalCoordinates([0., 0.0, 1.0]));

        let ab = CartesianVec3::from_point(a, b);

        let ac = CartesianVec3::from_point(a, c);

        let cross_prod = ab.cross(&ac);

        let normal = cross_prod.normalized();

        let expected_area = 0.5
            * (cross_prod.0[0].powi(2) + cross_prod.0[1].powi(2) + cross_prod.0[2].powi(2)).sqrt();

        let plane = make_bounded_plane(normal, a, 0);

        // let area = tetra_area([a, b, c, d], plane);

        let area =
            compute_intersection_area(&[a, b, c, d], VolumeElementTypes::Tetra4, &plane).unwrap();

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

        let normal = CartesianVec3([0., 0., 1.]);
        let point = CartesianCoordinates([0., 0., 0.5]);

        let plane = make_bounded_plane(normal, point, 2);

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

    ///Theta face at theta = 0, so the face lies in the plane y = 0 and r is measured along x
    fn theta_face(r: [f64; 2], z: [f64; 2]) -> BoundedPlane {
        BoundedPlane {
            normal: CartesianVec3([0., 1., 0.]),
            origin: CartesianCoordinates([1., 0., 0.]),
            extent_u: r,
            extent_v: z,
            axis: 1,
        }
    }

    ///Tetra whose cut by y = 0 is the triangle (r,z) = (1,0), (2,0), (1,1), of area 1/2
    ///   z
    ///   1 +
    ///     |\
    ///     | \        cut of the tetra by the face
    ///     |  \
    ///   0 +---+---> r
    ///     1   2
    fn tetra_cut_by_theta_face() -> [CartesianCoordinates; 4] {
        [
            CartesianCoordinates([1., -1., 0.]),
            CartesianCoordinates([3., -1., 0.]),
            CartesianCoordinates([1., -1., 2.]),
            CartesianCoordinates([1., 1., 0.]),
        ]
    }

    ///A polygon wrapping the whole annulus recovers its exact area
    #[test]
    fn test_annulus_area_of_a_surrounding_polygon() {
        let square = [[-5., -5., 0.], [5., -5., 0.], [5., 5., 0.], [-5., 5., 0.]];

        let area = polygon_annulus_area(&square, [1., 2.]);
        let expected = std::f64::consts::PI * (4. - 1.);

        assert!((area - expected).abs() < 1e-9, "{} != {}", area, expected);
    }

    ///A polygon inside the hole of the annulus carries no area
    #[test]
    fn test_annulus_area_inside_the_hole() {
        let square = [
            [-0.2, -0.2, 0.],
            [0.2, -0.2, 0.],
            [0.2, 0.2, 0.],
            [-0.2, 0.2, 0.],
        ];

        assert!(polygon_annulus_area(&square, [1., 2.]).abs() < 1e-12);
    }

    ///A polygon of the annulus itself keeps its plain area, the arcs cut nothing
    #[test]
    fn test_annulus_area_of_an_inner_polygon() {
        let patch = [
            [1.2, -0.1, 0.],
            [1.8, -0.1, 0.],
            [1.8, 0.1, 0.],
            [1.2, 0.1, 0.],
        ];

        let area = polygon_annulus_area(&patch, [1., 2.]);

        assert!((area - 0.12).abs() < 1e-9, "{} != 0.12", area);
    }

    #[test]
    fn test_area_inside_bounds_is_kept_whole() {
        let area = tetra_area(
            tetra_cut_by_theta_face(),
            &theta_face([0., 10.], [-10., 10.]),
        );

        assert!((area - 0.5).abs() < 1e-10, "expected 0.5, got {}", area);
    }

    ///An element straddling a bound used to be dropped, it now contributes its share.
    ///Cutting the triangle at z = 0.5 leaves a trapezoid of area 1/2 - 1/8
    #[test]
    fn test_area_straddling_a_bound_is_clipped() {
        let area = tetra_area(
            tetra_cut_by_theta_face(),
            &theta_face([0., 10.], [-10., 0.5]),
        );

        assert!((area - 0.375).abs() < 1e-10, "expected 0.375, got {}", area);
    }

    #[test]
    fn test_area_outside_bounds_is_dropped() {
        let area = tetra_area(
            tetra_cut_by_theta_face(),
            &theta_face([5., 10.], [-10., 10.]),
        );

        assert_eq!(area, 0.);
    }

    ///Patch of radius 1 spanning a 60 degree sector, tangent plane taken at its middle
    fn radial_patch() -> BoundedPlane {
        BoundedPlane {
            normal: CartesianVec3([1., 0., 0.]),
            origin: CartesianCoordinates([1., 0., 0.]),
            extent_u: [-0.5, 0.5],
            extent_v: [0., 1.],
            axis: RADIAL_AXIS,
        }
    }

    ///Small tetra straddling the cylinder r = 1 at theta = 0.4, far from the middle of the patch
    fn element_away_from_the_middle() -> [CartesianCoordinates; 4] {
        let theta = 0.4;
        let point = |r: f64, dtheta: f64, z: f64| {
            CartesianCoordinates([r * (theta + dtheta).cos(), r * (theta + dtheta).sin(), z])
        };
        [
            point(0.95, -0.02, 0.4),
            point(1.05, -0.02, 0.4),
            point(0.95, 0.02, 0.4),
            point(0.95, -0.02, 0.5),
        ]
    }

    #[test]
    fn test_tangent_plane_follows_the_element() {
        let element = element_away_from_the_middle();
        let centroid = CartesianCoordinates([
            element
                .iter()
                .map(|CartesianCoordinates(p)| p[0])
                .sum::<f64>()
                / 4.,
            element
                .iter()
                .map(|CartesianCoordinates(p)| p[1])
                .sum::<f64>()
                / 4.,
            element
                .iter()
                .map(|CartesianCoordinates(p)| p[2])
                .sum::<f64>()
                / 4.,
        ]);

        let patch = radial_patch();
        let plane = tangent_plane_at(&patch, centroid);

        //The plane stays on the cylinder and keeps the bounds of the patch
        let CartesianCoordinates(origin) = plane.origin;
        assert!((origin[0].hypot(origin[1]) - 1.).abs() < 1e-12);
        assert_eq!(plane.extent_u, patch.extent_u);
        assert_eq!(plane.axis, patch.axis);

        //The element is cut by its own tangent plane, the one of the compartment misses it
        let with_element_plane =
            compute_intersection_area(&element, VolumeElementTypes::Tetra4, &plane).unwrap();
        let with_patch_plane =
            compute_intersection_area(&element, VolumeElementTypes::Tetra4, &patch).unwrap();

        assert!(
            with_element_plane > 0.,
            "the element must be cut by its own tangent plane"
        );
        assert_eq!(
            with_patch_plane, 0.,
            "the tangent plane of the compartment does not reach this element"
        );
    }

    #[test]
    fn test_sort_points_ccw_3d() {
        let normal = CartesianVec3([0., 0., 1.]);
        let points = vec![[0.5, 0.0, 0.5], [0.0, 0.5, 0.5], [0.0, 0.0, 0.5]];
        let sorted = sort_points_ccw_3d(&points, &normal);
        assert_eq!(sorted[0], [0.0, 0.0, 0.5]);
        assert_eq!(sorted[1], [0.5, 0.0, 0.5]);
        assert_eq!(sorted[2], [0.0, 0.5, 0.5]);
    }
}
