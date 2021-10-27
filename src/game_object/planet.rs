use super::{Drawable, PhysicsObject, Spatial, Sprite};

use raylib::prelude::*;

use crate::math::Transform2D;

use rapier2d::prelude::*;

use rand::prelude::*;
use rand_pcg::Pcg64;

pub struct Planet {
    transform: Transform2D,
    physics_body: Option<RigidBodyHandle>,
    mass: f32,
    pub color_a: Color,
    pub color_b: Color,
    uuid: u128,
    sprite: Sprite,
}

impl Planet {
    #[allow(dead_code)]
    pub fn new(
        position: Vector2,
        rotation: f32,
        radius: f32,
        color_a: Color,
        color_b: Color,
        texture: WeakTexture2D,
    ) -> Self {
        let transform = Transform2D { position, rotation };
        let sprite = Sprite::new(texture, true, radius / 48.0);

        Planet {
            transform,
            physics_body: None,
            mass: 0.,
            color_a,
            color_b,
            uuid: Pcg64::from_entropy().gen(),
            sprite,
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
    fn draw(&self, rl: &mut RaylibShaderMode<RaylibMode2D<RaylibDrawHandle>>) {
        self.sprite.draw(rl, &self.transform)
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
