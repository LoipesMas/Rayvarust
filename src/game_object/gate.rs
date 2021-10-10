use super::{Drawable, GameObject, PhysicsObject, Spatial, Sprite};

use crate::{impl_drawable, impl_spatial};

use raylib::prelude::*;

use rapier2d::prelude::*;

pub struct Gate {
    pub game_object: GameObject,
    pub gate_num: u32,
}

pub const HIGHLIGHT_COLOR: Color = Color::new(250, 200, 200, 255);

impl Gate {
    pub fn new(texture: WeakTexture2D) -> Self {
        let mut game_object = GameObject::new();
        game_object.sprite = Some(Sprite::new(texture, true, 0.7));

        Self {
            game_object,
            gate_num: 0,
        }
    }
}

impl_spatial!(Gate);
impl_drawable!(Gate);

impl PhysicsObject for Gate {
    fn get_body(&self) -> &RigidBodyHandle {
        self.game_object.get_body()
    }

    fn set_body(&mut self, body: RigidBodyHandle) {
        self.game_object.set_body(body);
    }

    fn physics_process(&mut self, delta: f32, body: &mut RigidBody) {
        self.game_object.physics_process(delta, body);
    }

    fn update_state(&mut self, body: &RigidBody) {
        self.game_object.update_state(body);
    }
}
