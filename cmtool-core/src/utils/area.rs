// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{
    coordinates::*,
    ensight_gold::types::{ElementsType, VolumeElementTypes},
};

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

fn project_points_to_plane_2d(points: &Vec<[f64; 3]>, normal: &CartesianVec3) -> Vec<[f64; 2]> {
    let n = normal.normalized();
    let arbitrary = CartesianVec3::from_point_origin(if n.0[0].abs() < 0.9 {
        CartesianCoordinates([1.0, 0.0, 0.0])
    } else {
        CartesianCoordinates([0.0, 1.0, 0.0])
    });

    let u = n.cross(&arbitrary).normalized();
    let v = n.cross(&u);

    points
        .iter()
        .map(|p| CartesianVec3::from_point_origin(CartesianCoordinates(*p)))
        .map(|p| [p.dot(&u), p.dot(&v)])
        .collect()
}

fn sort_points_ccw_3d(points: &Vec<[f64; 3]>, normal: &CartesianVec3) -> Vec<[f64; 3]> {
    let projected = project_points_to_plane_2d(points, normal);
    let sorted_2d = sort_polygon_ccw(&projected);

    let mut sorted_3d = Vec::with_capacity(points.len());

    for p2d in &sorted_2d {
        let idx = projected
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                let da = (a[0] - p2d[0]).hypot(a[1] - p2d[1]);
                let db = (b[0] - p2d[0]).hypot(b[1] - p2d[1]);
                da.partial_cmp(&db).unwrap()
            })
            .map(|(i, _)| i)
            .unwrap();
        sorted_3d.push(points[idx]);
    }

    sorted_3d
}

// fn polygon_area_2d(points: &Vec<[f64; 2]>) -> f64 {
//     let n = points.len();
//     let mut area = 0.0;
//     for i in 0..n {
//         let (x0, y0) = (points[i][0], points[i][1]);
//         let (x1, y1) = (points[(i + 1) % n][0], points[(i + 1) % n][1]);
//         area += x0 * y1 - x1 * y0;
//     }
//     area.abs() * 0.5
// }

fn tetra_area(vertices: [CartesianCoordinates; 4], plane: &BoundedPlane) -> f64 {
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
    const TOL: f64 = 1e-1;
    for (i, dist) in distances.iter().enumerate() {
        if dist.abs() < TOL {
            points_on_plane.push(vertices[i].0);
        }
    }

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
        if points_on_plane.len() >= 3 {
            let filtered: Vec<_> = points_on_plane
                .iter()
                .cloned()
                .filter(|p| plane.is_point_inside(CartesianCoordinates(*p)))
                .collect();

            if filtered.len() >= 3 {
                let sorted = sort_points_ccw_3d(&filtered, normal);
                return polygon_area_3d(&sorted, normal);
            } else {
                return 0.0;
            }
        } else {
            return 0.0;
        }
    }

    //Project points to 2D plane
    // let projected = project_points_to_plane_2d(&intersection_points, normal);

    // //Sort points counterclockwise
    // let sorted = sort_polygon_ccw(&projected);

    let sorted = sort_points_ccw_3d(&intersection_points, normal);

    //Compute area using shoelace formula
    // return polygon_area_2d(&sorted);
    polygon_area_3d(&sorted, normal)
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
    fn make_bounded_plane(normal: CartesianVec3, origin: CartesianCoordinates) -> BoundedPlane {
        // let (u, v) = orthonormal_basis(&normal);

        let extent = [-10.0, 10.0];

        BoundedPlane {
            normal,
            origin,
            extent_u: extent,
            extent_v: extent,
            axis: 0,
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

        let plane = make_bounded_plane(normal, a);

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

        let plane = make_bounded_plane(normal, point);

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
}
