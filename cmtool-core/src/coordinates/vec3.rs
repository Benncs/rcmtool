use crate::coordinates::Coords3;


pub struct CartesianVec3(pub Coords3);

pub struct CylindricalVec3(pub Coords3);

pub trait Vec3 {
    fn sub(&self, other: &Self) -> Self;
    fn cross(&self, other: &Self) -> Self;
    fn dot(&self, other: &Self) -> f64;
    fn norm(&self)->f64;
    fn normalized(&self)->Self;
}


// impl Vec3 for CartesianVec3
// {
//     fn sub(&self, other: &Self) -> Self {
//         Self([self.0[0] - other.0[0], self.0[1] - other.0[1], self.0[2] - other.0[2]])
//     }

//     fn cross(&self, other: &Self) -> Self {
//         Self([f64::sqrt(self[0]*self[0]+self[1]*self[1]+self[2]*self[2])])
//     }

//     fn dot(&self, other: &Self) -> f64 {
//         todo!()
//     } 

//     fn norm(&self)->f64 {
//         todo!()
//     }

//     fn normalized(&self)->Self {
//         todo!()
//     }
// }

//https://en.wikipedia.org/wiki/Vector_fields_in_cylindrical_and_spherical_coordinates

impl From<CartesianVec3> for CylindricalVec3 {
    fn from(value: CartesianVec3) -> Self {
        let [x, y, z] = value.0;

        let r = (x.powi(2) + y.powi(2)).sqrt();
        let theta = y.atan2(x); // range [-π, π]
        
        CylindricalVec3([r, theta, z])
    }
}

impl From<CylindricalVec3> for CartesianVec3 {
    fn from(value: CylindricalVec3) -> Self {
        let [r, theta, z] = value.0;
        let x = r * theta.cos();
        let y = r * theta.sin();

        CartesianVec3([x, y, z])
    }
}
