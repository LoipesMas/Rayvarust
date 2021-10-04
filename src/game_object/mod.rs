use raylib::math::Vector2;
use raylib::prelude::*;

use crate::math::Transform2D;

mod sprite;
pub use sprite::Sprite;

mod player;
pub use player::Player;

use crate::physics::*;

pub trait Drawable {
    fn draw(&self, rl: &mut RaylibMode2D<RaylibDrawHandle>);
    fn get_scale(&self) -> f32;
    fn set_scale(&mut self, scale: f32);
}

pub trait Spatial {
    fn get_position(&self) -> Vector2;
    fn set_position(&mut self, position: Vector2);
    fn translate(&mut self, vector: Vector2);
}

pub trait Processing {
    fn process(&mut self, rl: &mut RaylibHandle, delta: f32);
}

pub trait PhysicsObject {
    fn get_body(&self) -> &PhysicsBody;
    fn get_body_mut(&mut self) -> &mut PhysicsBody;
    fn physics_process(&mut self, delta: f32);
}

pub struct GameObject {
    transform: Transform2D,
    pub sprite: Option<Sprite>,
    pub physics_body: Option<PhysicsBody>,
}

impl GameObject {
    pub fn new() -> GameObject {
        let transform = Transform2D {
            position: Vector2::zero(),
            rotation: 0.0,
        };
        GameObject {
            transform,
            sprite: None,
            physics_body: None,
        }
    }
}

impl Drawable for GameObject {
    fn draw(&self, rl: &mut RaylibMode2D<RaylibDrawHandle>) {
        if let Some(s) = &self.sprite {
            s.draw(rl, &self.transform);
        }
    }

    fn get_scale(&self) -> f32 {
        if let Some(s) = &self.sprite {
            s.get_scale()
        } else {
            1.0
        }
    }
    fn set_scale(&mut self, scale: f32) {
        if let Some(s) = self.sprite.as_mut() {
            s.set_scale(scale)
        }
    }
}

impl Spatial for GameObject {
    fn get_position(&self) -> Vector2 {
        self.transform.position
    }

    fn set_position(&mut self, position: Vector2) {
        self.transform.position = position;
        if let Some(b) = self.physics_body.as_mut() {
            b.set_position(position);
        }
    }

    fn translate(&mut self, vector: Vector2) {
        self.set_position(self.transform.position + vector);
    }
}

impl Processing for GameObject {
    #[allow(unused_variables)]
    fn process(&mut self, rl: &mut RaylibHandle, delta: f32) {
        let x = f32::sqrt(delta);
    }
}

impl PhysicsObject for GameObject {
    fn get_body(&self) -> &PhysicsBody {
        self.physics_body
            .as_ref()
            .expect("Tried to get PhysicsBody of non-physic object")
    }

    fn get_body_mut(&mut self) -> &mut PhysicsBody {
        self.physics_body
            .as_mut()
            .expect("Tried to get PhysicsBody of non-physic object")
    }

    fn physics_process(&mut self, delta: f32) {
        self.translate(self.get_body().get_linear_velocity() * delta);
    }
}
