use raylib::math::Vector2;
use raylib::prelude::*;

use crate::math::Transform2D;

mod utils;

mod sprite;
pub use sprite::Sprite;

mod player;
pub use player::Player;

mod planet;
pub use planet::Planet;

mod gate;
pub use gate::{Gate, HIGHLIGHT_COLOR};

use rapier2d::prelude::*;

use rand::prelude::*;
use rand_pcg::Pcg64;

pub struct GameObject {
    uuid: u128,
    transform: Transform2D,
    pub sprite: Option<Sprite>,
    pub physics_body: Option<RigidBodyHandle>,
}

impl GameObject {
    pub fn new() -> GameObject {
        let transform = Transform2D {
            position: Vector2::zero(),
            rotation: 0.0,
        };
        GameObject {
            uuid: Pcg64::from_entropy().gen(),
            transform,
            sprite: None,
            physics_body: None,
        }
    }

    pub fn get_uuid(&self) -> u128 {
        self.uuid
    }
}

pub trait Drawable {
    fn draw(&self, rl: &mut RaylibMode2D<RaylibDrawHandle>);
    fn get_scale(&self) -> f32;
    fn set_scale(&mut self, scale: f32);
    fn set_tint(&mut self, tint: Color);
}

pub trait Spatial {
    fn get_position(&self) -> Vector2;
    fn set_position(&mut self, position: Vector2);
    fn get_rotation(&self) -> f32;
    fn set_rotation(&mut self, rotation: f32);
    fn translate(&mut self, vector: Vector2);
}

pub trait Processing {
    fn process(&mut self, rl: &mut RaylibHandle, delta: f32);
}

pub trait PhysicsObject {
    fn get_body(&self) -> &RigidBodyHandle;
    fn set_body(&mut self, body: RigidBodyHandle);
    fn physics_process(&mut self, delta: f32, body: &mut RigidBody);
    fn update_state(&mut self, body: &RigidBody);
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
            s.set_scale(scale);
        }
    }

    fn set_tint(&mut self, tint: Color) {
        if let Some(s) = self.sprite.as_mut() {
            s.set_tint(tint);
        }
    }
}

impl Spatial for GameObject {
    fn get_position(&self) -> Vector2 {
        self.transform.position
    }

    fn set_position(&mut self, position: Vector2) {
        self.transform.position = position;
    }

    fn get_rotation(&self) -> f32 {
        self.transform.rotation
    }

    fn set_rotation(&mut self, rotation: f32) {
        self.transform.rotation = rotation;
    }
    fn translate(&mut self, vector: Vector2) {
        self.set_position(self.transform.position + vector);
    }
}

impl Processing for GameObject {
    #[allow(unused_variables)]
    fn process(&mut self, rl: &mut RaylibHandle, delta: f32) {}
}

impl PhysicsObject for GameObject {
    fn get_body(&self) -> &RigidBodyHandle {
        self.physics_body
            .as_ref()
            .expect("Tried to get PhysicsBody of non-physic object")
    }

    fn set_body(&mut self, body: RigidBodyHandle) {
        self.physics_body = Some(body);
    }

    fn update_state(&mut self, body: &RigidBody) {
        let pos = Vector2 {
            x: body.translation().x,
            y: body.translation().y,
        };
        let rot = body.rotation().angle();
        self.set_position(pos);
        self.transform.rotation = rot;
    }

    #[allow(unused_variables)]
    fn physics_process(&mut self, delta: f32, body: &mut RigidBody) {}
}
