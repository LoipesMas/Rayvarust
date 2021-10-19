use super::{Drawable, PhysicsObject, Spatial};

use raylib::prelude::*;

use crate::math::Transform2D;

use rapier2d::prelude::*;

use rand::prelude::*;
use rand_pcg::Pcg64;

pub struct Planet {
    transform: Transform2D,
    radius: f32,
    physics_body: Option<RigidBodyHandle>,
    mass: f32,
    color: Color,
    uuid: u128,
}

impl Planet {
    #[allow(dead_code)]
    pub fn new(position: Vector2, rotation: f32, radius: f32, color: Color) -> Self {
        let transform = Transform2D { position, rotation };

        Planet {
            transform,
            radius,
            physics_body: None,
            mass: 0.,
            color,
            uuid: Pcg64::from_entropy().gen(),
        }
    }

    pub fn get_mass(&self) -> f32 {
        self.mass
    }

    pub fn get_uuid(&self) -> u128 {
        self.uuid
    }
}

impl Spatial for Planet {
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
        self.set_position(self.get_position() + vector);
    }
}

impl Drawable for Planet {
    fn draw(&self, rl: &mut RaylibMode2D<RaylibDrawHandle>) {
        rl.draw_circle_v(self.get_position(), self.radius, self.color);
    }

    fn get_scale(&self) -> f32 {
        1.0
    }

    #[allow(unused_variables)]
    fn set_scale(&mut self, scale: f32) {}
    #[allow(unused_variables)]
    fn set_tint(&mut self, tint: Color) {}
}

impl PhysicsObject for Planet {
    fn get_body(&self) -> &RigidBodyHandle {
        self.physics_body
            .as_ref()
            .expect("Tried to get PhysicsBody of non-physic object")
    }

    fn set_body(&mut self, body: RigidBodyHandle) {
        self.physics_body = Some(body);
    }

    #[allow(unused_variables)]
    fn physics_process(&mut self, delta: f32, body: &mut RigidBody) {}

    fn update_state(&mut self, body: &RigidBody) {
        let pos = Vector2 {
            x: body.translation().x,
            y: body.translation().y,
        };
        let rot = body.rotation().angle();
        self.mass = body.mass();
        self.set_position(pos);
        self.transform.rotation = rot;
    }
}
