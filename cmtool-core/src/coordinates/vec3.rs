// SPDX-License-Identifier: GPL-3.0-or-later

use crate::coordinates::{CartesianCoordinates, Coords3};

#[derive(Clone, Copy)]
pub struct CartesianVec3(pub Coords3);

#[derive(Clone, Copy)]
pub struct CylindricalVec3(pub Coords3, pub f64);

pub trait Vec3 {
    fn sub(&self, other: &Self) -> Self;
    fn add(&self, other: &Self) -> Self;
    fn cross(&self, other: &Self) -> Self;
    fn dot(&self, other: &Self) -> f64;
    fn norm(&self) -> f64;
    fn normalized(&self) -> Self;
    fn scale(&self, lambda: f64) -> Self;
}

impl CartesianVec3 {
    pub fn from_point(a: CartesianCoordinates, b: CartesianCoordinates) -> Self {
        Self([b.0[0] - a.0[0], b.0[1] - a.0[1], b.0[2] - a.0[2]])
    }

    pub fn from_point_origin(a: CartesianCoordinates) -> Self {
        Self(a.0)
    }
}

impl Vec3 for CartesianVec3 {
    fn add(&self, other: &Self) -> Self {
        Self([
            self.0[0] + other.0[0],
            self.0[1] + other.0[1],
            self.0[2] + other.0[2],
        ])
    }

    fn sub(&self, other: &Self) -> Self {
        Self([
            self.0[0] - other.0[0],
            self.0[1] - other.0[1],
            self.0[2] - other.0[2],
        ])
    }

    fn scale(&self, lambda: f64) -> Self {
        Self(self.0.map(|v| v * lambda))
    }

    fn cross(&self, other: &Self) -> Self {
        Self([
            self.0[1] * other.0[2] - self.0[2] * other.0[1],
            self.0[2] * other.0[0] - self.0[0] * other.0[2],
            self.0[0] * other.0[1] - self.0[1] * other.0[0],
        ])
    }

    fn dot(&self, other: &Self) -> f64 {
        self.0[0] * other.0[0] + self.0[1] * other.0[1] + self.0[2] * other.0[2]
    }

    fn norm(&self) -> f64 {
        f64::sqrt(self.0[0] * self.0[0] + self.0[1] * self.0[1] + self.0[2] * self.0[2])
    }

    fn normalized(&self) -> Self {
        let _norm = self.norm();
        self.scale(1. / _norm)
    }
}

//https://en.wikipedia.org/wiki/Vector_fields_in_cylindrical_and_spherical_coordinates
impl CylindricalVec3 {
    pub fn from_cartesian_vec(value: CartesianVec3, base_theta: f64) -> Self {
        let vx = value.0[0];
        let vy = value.0[1];
        let vz = value.0[2];

        let vr = vx * base_theta.cos() + vy * base_theta.sin();
        let vtheta = -vx * base_theta.sin() + vy * base_theta.cos();

        CylindricalVec3([vr, vtheta, vz], base_theta)
    }

    pub fn to_cartesian_vec(&self) -> CartesianVec3 {
        let vr = self.0[0];
        let vtheta = self.0[1];
        let vz = self.0[2];
        let base_theta = self.1;

        let vx = vr * base_theta.cos() - vtheta * base_theta.sin();
        let vy = vr * base_theta.sin() + vtheta * base_theta.cos();

        CartesianVec3([vx, vy, vz])
    }
}

impl CartesianVec3 {
    pub fn from_cylindrical_vec(value: CylindricalVec3) -> Self {
        let vr = value.0[0];
        let vtheta = value.0[1];
        let vz = value.0[2];
        let base_theta = value.1;

        let vx = vr * base_theta.cos() - vtheta * base_theta.sin();
        let vy = vr * base_theta.sin() + vtheta * base_theta.cos();

        CartesianVec3([vx, vy, vz])
    }

    pub fn to_cylindrical_vec(&self, base_theta: f64) -> CylindricalVec3 {
        let vx = self.0[0];
        let vy = self.0[1];
        let vz = self.0[2];

        let vr = vx * base_theta.cos() + vy * base_theta.sin();
        let vtheta = -vx * base_theta.sin() + vy * base_theta.cos();

        CylindricalVec3([vr, vtheta, vz], base_theta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cartesian_cylindrical_round_trip() {
        let original = CartesianVec3([3.0, 4.0, 5.0]);
        let cyl = original.to_cylindrical_vec(0.);
        let converted = cyl.to_cartesian_vec();

        for i in 0..3 {
            assert!((converted.0[i] - original.0[i]).abs() < 1e-10);
        }
    }

    #[test]
    fn test_cartesian_cylindrical_1() {
        let base_theta = 0.0;

        let cart = CartesianVec3([3.0, 4.0, 5.0]);
        let cyl = cart.to_cylindrical_vec(base_theta);

        let vr_expected = 3.0;
        let vtheta_expected = 4.0;
        let vz_expected = 5.0;

        assert!((cyl.0[0] - vr_expected).abs() < 1e-10);
        assert!((cyl.0[1] - vtheta_expected).abs() < 1e-10);
        assert!((cyl.0[2] - vz_expected).abs() < 1e-10);
        assert!((cyl.1 - base_theta).abs() < 1e-10);

        // Conversion inverse
        let cart_converted = cyl.to_cartesian_vec();

        for i in 0..3 {
            assert!((cart_converted.0[i] - cart.0[i]).abs() < 1e-10);
        }
    }

    #[test]
    fn test_cartesian_to_cylindrical() {
        let base_theta = std::f64::consts::FRAC_PI_4;

        let cart = CartesianVec3([1.0, 0.0, 2.0]);
        let cyl = cart.to_cylindrical_vec(base_theta);

        let vr_expected = 1.0 * base_theta.cos() + 0.0 * base_theta.sin();
        let vtheta_expected = -1.0 * base_theta.sin() + 0.0 * base_theta.cos();
        let vz_expected = 2.0;

        assert!((cyl.0[0] - vr_expected).abs() < 1e-10);
        assert!((cyl.0[1] - vtheta_expected).abs() < 1e-10);
        assert!((cyl.0[2] - vz_expected).abs() < 1e-10);

        // Puis conversion inverse
        let cart_converted = cyl.to_cartesian_vec();

        for i in 0..3 {
            assert!((cart_converted.0[i] - cart.0[i]).abs() < 1e-10);
        }
    }

    #[test]
    fn test_cylindrical_to_cartesian_base_theta_0() {
        let r = 2.0_f64.sqrt();
        let theta = std::f64::consts::FRAC_PI_4;
        let z = 3.0;
        let base_theta = 0.0;

        let cyl = CylindricalVec3([r, theta, z], base_theta);
        let cart = cyl.to_cartesian_vec();
        // vx = vr * cos(0) - vtheta * sin(0) = vr
        // vy = vr * sin(0) + vtheta * cos(0) = vtheta
        let x_expected = r;
        let y_expected = theta;
        let z_expected = 3.0;

        assert!((cart.0[0] - x_expected).abs() < 1e-10);
        assert!((cart.0[1] - y_expected).abs() < 1e-10);
        assert!((cart.0[2] - z_expected).abs() < 1e-10);
    }
    #[test]
    fn test_sub() {
        let v1 = CartesianVec3([3.0, 4.0, 5.0]);
        let v2 = CartesianVec3([1.0, 1.0, 1.0]);
        let result = v1.sub(&v2);
        assert_eq!(result.0, [2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_scale() {
        let v = CartesianVec3([1.0, -2.0, 3.0]);
        let result = v.scale(2.0);
        assert_eq!(result.0, [2.0, -4.0, 6.0]);
    }

    #[test]
    fn test_dot() {
        let v1 = CartesianVec3([1.0, 2.0, 3.0]);
        let v2 = CartesianVec3([4.0, -5.0, 6.0]);
        let result = v1.dot(&v2);
        assert_eq!(result, 12.0); // 1*4 + 2*(-5) + 3*6 = 4 -10 +18 = 12
    }

    #[test]
    fn test_cross() {
        let v1 = CartesianVec3([1.0, 0.0, 0.0]);
        let v2 = CartesianVec3([0.0, 1.0, 0.0]);
        let result = v1.cross(&v2);
        assert_eq!(result.0, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_norm() {
        let v = CartesianVec3([3.0, 4.0, 0.0]);
        let result = v.norm();
        assert!((result - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_normalized() {
        let v = CartesianVec3([0.0, 3.0, 4.0]);
        let normed = v.normalized();
        let expected = [0.0, 0.6, 0.8];
        for i in 0..3 {
            assert!((normed.0[i] - expected[i]).abs() < 1e-10);
        }
    }
}
