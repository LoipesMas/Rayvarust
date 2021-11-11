use super::{Drawable, GameObject, PhysicsObject, Spatial, Sprite, Transform2D};

use crate::{impl_drawable, impl_spatial, DrawHandle};

use raylib::prelude::*;

use rapier2d::prelude::*;

pub struct Gate {
    pub game_object: GameObject,
    pub gate_num: u32,
    tex: WeakTexture2D,
    off_tex: WeakTexture2D,
    is_off: bool,
}

pub const HIGHLIGHT_COLOR: Color = Color::new(255, 255, 255, 255);

impl Gate {
    pub fn new(tex: WeakTexture2D, off_tex: WeakTexture2D) -> Self {
        let mut game_object = GameObject::new();
        game_object.sprite = Some(Sprite::new(tex.clone(), true, 0.7));

        Self {
            game_object,
            gate_num: 0,
            tex,
            off_tex,
            is_off: false,
        }
    }

    pub fn get_uuid(&self) -> u128 {
        self.game_object.get_uuid()
    }

    pub fn set_off(&mut self, value: bool) {
        println!("{}", value);
        if value == self.is_off {
            return;
        }
        self.is_off = value;

        let tex = if self.is_off {
            self.off_tex.clone()
        } else {
            self.tex.clone()
        };

        self.game_object.sprite.as_mut().unwrap().set_texture(tex);
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
