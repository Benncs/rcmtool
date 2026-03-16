// SPDX-License-Identifier: GPL-3.0-or-later

mod points;
mod vec3;
pub use points::*;
pub use vec3::*;

pub const NUMBER_OF_AXIS: usize = 3;
pub type Coords3 = [f64; NUMBER_OF_AXIS];

pub fn orthonormal_basis(normal: &CartesianVec3) -> (CartesianVec3, CartesianVec3) {
    let n = normal.normalized();
    let temp = if n.0[0].abs() < 0.9 {
        CartesianVec3([1.0, 0.0, 0.0])
    } else {
        CartesianVec3([0.0, 1.0, 0.0])
    };

    let u = n.cross(&temp).normalized();
    let v = n.cross(&u).normalized();
    (u, v)
}

pub struct Plane {
    pub normal: CartesianVec3,
    pub point: CartesianCoordinates,
}

pub struct BoundedPlane {
    pub normal: CartesianVec3,
    pub origin: CartesianCoordinates,
    pub extent_u: [f64; 2],
    pub extent_v: [f64; 2],
    pub axis: usize, // 0 = r, 1 = theta, 2 = z
}

impl BoundedPlane {
    pub fn is_point_inside(&self, CartesianCoordinates(point): CartesianCoordinates) -> bool {
        let r = (point[0].powi(2) + point[1].powi(2)).sqrt();
        let theta_raw = point[1].atan2(point[0]);
        let z = point[2];
        let tol = 1e-10;

        let (u, v) = match self.axis {
            0 => {
                let pi2 = 2.0 * std::f64::consts::PI;
                let mid = 0.5 * (self.extent_u[0] + self.extent_u[1]);
                let mut theta = theta_raw;
                while theta - mid > std::f64::consts::PI {
                    theta -= pi2;
                }
                while mid - theta > std::f64::consts::PI {
                    theta += pi2;
                }
                (theta, z)
            }
            1 => (r, z),
            2 => {
                let pi2 = 2.0 * std::f64::consts::PI;
                let mid = 0.5 * (self.extent_v[0] + self.extent_v[1]);
                let mut theta = theta_raw;
                while theta - mid > std::f64::consts::PI {
                    theta -= pi2;
                }
                while mid - theta > std::f64::consts::PI {
                    theta += pi2;
                }
                (r, theta)
            }
            _ => unreachable!(),
        };

        (u >= self.extent_u[0] - tol && u <= self.extent_u[1] + tol)
            && (v >= self.extent_v[0] - tol && v <= self.extent_v[1] + tol)
    }
}

impl From<BoundedPlane> for Plane {
    fn from(value: BoundedPlane) -> Self {
        Self {
            normal: value.normal,
            point: value.origin,
        }
    }
}

// pub trait AsCoords3 {
//     fn as_coords(&self) -> &Coords3;
// }

// impl AsCoords3 for CylindricalCoordinates {
//     fn as_coords(&self) -> &Coords3 {
//         &self.0
//     }
// }

// impl AsCoords3 for CartesianCoordinates {
//     fn as_coords(&self) -> &Coords3 {
//         &self.0
//     }
// }

// impl AsCoords3 for Coords3 {
//     fn as_coords(&self) -> &Coords3 {
//         self
//     }
// }
