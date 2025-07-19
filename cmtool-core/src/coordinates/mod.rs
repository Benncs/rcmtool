mod points;
mod vec3;
pub use points::*;
pub use vec3::*;

pub const NUMBER_OF_AXIS: usize = 3;
pub type Coords3 = [f64; NUMBER_OF_AXIS];

pub trait Coords3Ext {
    fn sub(&self, other: &Self) -> Self;
    fn cross(&self, other: &Self) -> Self;
    fn dot(&self, other: &Self) -> f64;
    fn norm(&self) -> f64;
    fn normalized(&self) -> Self;
}

impl Coords3Ext for Coords3 {
    fn sub(&self, other: &Self) -> Self {
        [self[0] - other[0], self[1] - other[1], self[2] - other[2]]
    }

    fn norm(&self) -> f64 {
        f64::sqrt(self[0] * self[0] + self[1] * self[1] + self[2] * self[2])
    }

    fn normalized(&self) -> Self {
        let _norm = self.norm();
        [self[0] / _norm, self[1] / _norm, self[2] / _norm]
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

pub struct Plane {
    pub normal: CartesianVec3,
    pub point: CartesianCoordinates,
}

pub struct BoundedPlane {
    pub normal: CartesianVec3,
    pub origin: CartesianCoordinates,
    pub extent_u: [f64; 2],
    pub extent_v: [f64; 2],
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
        let v = CartesianVec3([
            point[0] - self.origin.0[0],
            point[1] - self.origin.0[1],
            point[2] - self.origin.0[2],
        ]);

        let (u_dir, v_dir) = orthonormal_basis(&self.normal);

        let u_proj = v.dot(&u_dir);
        let v_proj = v.dot(&v_dir);

        (u_proj >= self.extent_u[0] && u_proj <= self.extent_u[1])
            && (v_proj >= self.extent_v[0] && v_proj <= self.extent_v[1])
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

pub trait AsCoords3 {
    fn as_coords(&self) -> &Coords3;
}

impl AsCoords3 for CylindricalCoordinates {
    fn as_coords(&self) -> &Coords3 {
        &self.0
    }
}

impl AsCoords3 for CartesianCoordinates {
    fn as_coords(&self) -> &Coords3 {
        &self.0
    }
}

impl AsCoords3 for Coords3 {
    fn as_coords(&self) -> &Coords3 {
        self
    }
}
