use super::{Drawable, GameObject, PhysicsObject, Spatial, Sprite, Transform2D};

use crate::{impl_drawable, impl_spatial, DrawHandle};

use raylib::prelude::*;

use rapier2d::prelude::*;

pub struct Gate {
    pub game_object: GameObject,
    pub gate_num: u32,
    tex: WeakTexture2D,
    off_tex: WeakTexture2D,
    darker_tex: WeakTexture2D,
    is_off: bool,
    highlight: bool,
}

impl Gate {
    pub fn new(tex: WeakTexture2D, off_tex: WeakTexture2D, darker_tex: WeakTexture2D) -> Self {
        let mut game_object = GameObject::new();
        game_object.sprite = Some(Sprite::new(darker_tex.clone(), true, 0.7));

        Self {
            game_object,
            gate_num: 0,
            tex,
            off_tex,
            darker_tex,
            is_off: false,
            highlight: false,
        }
    }

    pub fn get_uuid(&self) -> u128 {
        self.game_object.get_uuid()
    }

    pub fn set_state(&mut self, is_off: bool, highlight: bool) {
        if is_off == self.is_off && highlight == self.highlight {
            return;
        }
        self.is_off = is_off;
        self.highlight = highlight;

        let tex = if self.is_off {
            self.off_tex.clone()
        } else if self.highlight {
            self.tex.clone()
        } else {
            self.darker_tex.clone()
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
