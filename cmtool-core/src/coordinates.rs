pub const NUMBER_OF_AXIS: usize = 3;
pub type Coords3 = [f64; NUMBER_OF_AXIS];

pub trait Coords3Ext {
    fn sub(&self, other: &Self) -> Self;
    fn cross(&self, other: &Self) -> Self;
    fn dot(&self, other: &Self) -> f64;

    fn cylindrical_to_cartesian(&self) -> Self;
    fn cartesian_to_cylindrical(&self) -> Self;
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

#[derive(Debug,Clone, Copy,Default)]
pub struct CylindricalCoordinates(pub Coords3);
#[derive(Debug,Clone, Copy,Default)]
pub struct CartesianCoordinates(pub Coords3);

impl From<&CartesianCoordinates> for CylindricalCoordinates {
    fn from(cart: &CartesianCoordinates) -> Self {
        CylindricalCoordinates(cart.0.cartesian_to_cylindrical())
    }
}

impl From<CartesianCoordinates> for CylindricalCoordinates {
    fn from(cart: CartesianCoordinates) -> Self {
        CylindricalCoordinates::from(&cart)
    }
}

impl From<&CylindricalCoordinates> for CartesianCoordinates {
    fn from(value: &CylindricalCoordinates) -> Self {
        CartesianCoordinates(value.0.cylindrical_to_cartesian())
    }
}

impl From<CylindricalCoordinates> for CartesianCoordinates {
    fn from(value: CylindricalCoordinates) -> Self {
        CartesianCoordinates::from(&value)
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