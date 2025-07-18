use super::Coords3;


#[derive(Debug, Clone, Copy, Default)]
pub struct CylindricalCoordinates(pub Coords3);
#[derive(Debug, Clone, Copy, Default)]
pub struct CartesianCoordinates(pub Coords3);


impl From<&CartesianCoordinates> for CylindricalCoordinates {
    fn from(cart: &CartesianCoordinates) -> Self {
        let r = f64::sqrt(cart.0[0] * cart.0[0] + cart.0[1] * cart.0[1]);
        let theta = cart.0[1].atan2(cart.0[0]); // handles all cases
        Self([r, theta, cart.0[2]])
    }
}

impl From<CartesianCoordinates> for CylindricalCoordinates {
    fn from(cart: CartesianCoordinates) -> Self {
        CylindricalCoordinates::from(&cart)
    }
}

impl From<&CylindricalCoordinates> for CartesianCoordinates {
    fn from(value: &CylindricalCoordinates) -> Self {
        CartesianCoordinates( [value.0[0] * value.0[1].cos(), value.0[0] * value.0[1].sin(), value.0[2]])
    }
}

impl From<CylindricalCoordinates> for CartesianCoordinates {
    fn from(value: CylindricalCoordinates) -> Self {
        CartesianCoordinates::from(&value)
    }
}