use super::{Drawable, GameObject, PhysicsObject, Spatial, Sprite};

use raylib::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

use crate::{impl_drawable, impl_spatial};

use rapier2d::prelude::*;

pub struct Planet {
    game_object: GameObject,
}

impl Planet {
    #[allow(dead_code)]
    pub fn new(texture: Rc<RefCell<WeakTexture2D>>) -> Self {
        let mut game_object = GameObject::new();
        game_object.sprite = Some(Sprite::new(texture, true, 0.7));

        // TODO: use rapier2d to create rigidbody
        //let phys_body = PhysicsBody::new(CollisionShape::Circle(Vector2::zero(), 40.));
        //game_object.physics_body = Some(phys_body);

        Planet { game_object }
    }
}

impl_spatial!(Planet);
impl_drawable!(Planet);

impl PhysicsObject for Planet {
    fn get_body(&self) -> &RigidBodyHandle {
        self.game_object.get_body()
    }

    fn get_body_mut(&mut self) -> &mut RigidBodyHandle {
        self.game_object.get_body_mut()
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
