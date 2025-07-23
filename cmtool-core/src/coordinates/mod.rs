mod points;
mod vec3;
pub use points::*;
pub use vec3::*;

pub const NUMBER_OF_AXIS: usize = 3;
pub type Coords3 = [f64; NUMBER_OF_AXIS];

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

impl BoundedPlane {
    pub fn is_point_inside(&self, CartesianCoordinates(point): CartesianCoordinates) -> bool {
        // let v = CartesianVec3([
        //     point[0] - self.origin.0[0],
        //     point[1] - self.origin.0[1],
        //     point[2] - self.origin.0[2],
        // ]);

        // let (u_dir, v_dir) = orthonormal_basis(&self.normal);

        // let u_proj = v.dot(&u_dir);
        // let v_proj = v.dot(&v_dir);

        // (u_proj >= self.extent_u[0] && u_proj <= self.extent_u[1])
        //     && (v_proj >= self.extent_v[0] && v_proj <= self.extent_v[1])

        let r = (point[0].powi(2) + point[1].powi(2)).sqrt();
        let theta = point[1].atan2(point[0]);
        let z = point[2];

        let (u, v) = match self.axis {
            0 => {
                let u = theta;
                let v = z;
                (u, v)
            }
            1 => {
                let u = r;
                let v = z;
                (u, v)
            }
            2 => {
                let u = r;
                let v = theta;
                (u, v)
            }
            _ => unreachable!(),
        };
        (u >= self.extent_u[0] && u <= self.extent_u[1])
            && (v >= self.extent_v[0] && v <= self.extent_v[1])
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
