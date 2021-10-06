#![allow(dead_code)]

use raylib::math::Vector2;

use rapier2d::prelude::Real;
/// Vector2 from nalgebra (so it doesnt collide with raylib's Vector2)
pub type NVector2 = rapier2d::prelude::nalgebra::base::Vector2<Real>;

use std::ops::{Add, Mul, Sub};

pub struct Transform2D {
    pub position: Vector2,
    pub rotation: f32,
}

pub fn lerp<T: Sub<Output = T> + Clone, U>(x: T, y: T, s: U) -> T
where
    U: Mul<T>,
    T: Mul<U, Output = T> + Add<T, Output = T>,
{
    x.clone() + (y - x) * s
}

/// Converts nalgebra Vector2 to raylib's Vector2
pub fn to_rv2(nvec: NVector2) -> Vector2 {
    Vector2 {
        x: nvec[0],
        y: nvec[1],
    }
}

/// Converts raylib's Vector2 to nalgebra Vector2
pub fn to_nv2(rvec: Vector2) -> NVector2 {
    let mut result = NVector2::zeros();
    result.x = rvec.x;
    result.y = rvec.y;
    result
}
