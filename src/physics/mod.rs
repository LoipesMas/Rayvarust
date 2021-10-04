use crate::check_collision_circles;
use crate::Vector2;
use raylib::prelude::*;

use crate::CollisionShape::*;

#[allow(dead_code)]
pub enum CollisionShape {
    Circle(Vector2, f32),
}

const DRAW_COLOR: Color = Color {
    r: 90,
    g: 70,
    b: 130,
    a: 200,
};

pub struct PhysicsBody {
    shape: CollisionShape,
    linear_velocity: Vector2,
    anuglar_velocity: f32,
}

impl PhysicsBody {
    pub fn new(shape: CollisionShape) -> PhysicsBody {
        PhysicsBody {
            shape,
            linear_velocity: Vector2::zero(),
            anuglar_velocity: 0.0,
        }
    }

    pub fn debug_draw(&self, rl: &mut RaylibMode2D<RaylibDrawHandle>) {
        match self.shape {
            Circle(p, r) => {
                rl.draw_circle_v(p, r, DRAW_COLOR);
            }
        }
    }

    pub fn set_position(&mut self, position: Vector2) {
        match self.shape {
            Circle(_, r) => self.shape = Circle(position, r),
        }
    }

    pub fn get_linear_velocity(&self) -> Vector2 {
        self.linear_velocity
    }

    pub fn set_linear_velocity(&mut self, velocity: Vector2) {
        self.linear_velocity = velocity;
    }

    pub fn add_linear_velocity(&mut self, velocity: Vector2) {
        self.set_linear_velocity(self.linear_velocity + velocity);
    }

    pub fn get_angular_velocity(&self) -> f32 {
        self.anuglar_velocity
    }

    pub fn set_angular_velocity(&mut self, velocity: f32) {
        self.anuglar_velocity = velocity;
    }

    pub fn add_angular_velocity(&mut self, velocity: f32) {
        self.set_angular_velocity(self.anuglar_velocity + velocity);
    }

    pub fn check_body_collision(&self, other: &PhysicsBody) -> bool {
        match self.shape {
            CollisionShape::Circle(p, r) => match other.shape {
                CollisionShape::Circle(p_o, r_o) => check_collision_circles(p, r, p_o, r_o),
            },
        }
    }
}
