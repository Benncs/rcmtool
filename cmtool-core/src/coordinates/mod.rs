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
    fn norm(&self)->f64;
    fn normalized(&self)->Self;

}

impl Coords3Ext for Coords3 {
    fn sub(&self, other: &Self) -> Self {
        [self[0] - other[0], self[1] - other[1], self[2] - other[2]]
    }
    

    fn norm(&self)->f64
    {
        f64::sqrt(self[0]*self[0]+self[1]*self[1]+self[2]*self[2])
    }

    fn normalized(&self)->Self
    {
        let _norm = self.norm();
        [self[0]/_norm,self[1]/_norm,self[2]/_norm]
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



pub enum Plane {
    Vector {
        normal: CartesianVec3,
        point: CartesianCoordinates,
    },
    Cartesian {
        normal: Coords3,
        d: f64,
    },
    Cylindrical {
        axis: usize,     // 0=r, 1=theta, 2=z
        coordinate: f64, // plane position along axis
        theta: f64,      // required for axis 0 and 1
        radius: f64,     // required for axis 1
    },
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
