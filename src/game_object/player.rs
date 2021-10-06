use super::{Drawable, GameObject, PhysicsObject, Processing, Spatial, Sprite};

use crate::{impl_drawable, impl_spatial};

use raylib::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

use crate::math::NVector2;

use rapier2d::prelude::*;

pub struct Player {
    game_object: GameObject,
    pub lin_speed: f32,
    pub ang_speed: f32,
    move_vec: NVector2, // add this to lin vel on next phys process
    rot: f32,           // add this to ang vel on next phys process
}

#[allow(dead_code)]
impl Player {
    pub fn new(texture: Rc<RefCell<WeakTexture2D>>) -> Self {
        let mut game_object = GameObject::new();
        game_object.sprite = Some(Sprite::new(texture, true, 0.7));

        // TODO: use rapier2d to create rigidbody
        //let phys_body = PhysicsBody::new(CollisionShape::Circle(Vector2::zero(), 40.));
        //game_object.physics_body = Some(phys_body);

        Player {
            game_object,
            lin_speed: 30.0,
            ang_speed: 1.0,
            move_vec: NVector2::zeros(),
            rot: 0.0,
        }
    }
}

impl_spatial!(Player);
impl_drawable!(Player);

impl Processing for Player {
    #[allow(unused_variables)]
    fn process(&mut self, rl: &mut RaylibHandle, delta: f32) {
        let move_u = rl.is_key_down(KeyboardKey::KEY_W);
        let move_d = rl.is_key_down(KeyboardKey::KEY_S);
        let move_l = rl.is_key_down(KeyboardKey::KEY_A);
        let move_r = rl.is_key_down(KeyboardKey::KEY_D);

        self.move_vec = vector![0., 0.];

        if move_u {
            self.move_vec.y -= self.lin_speed * 4.0;
        }
        if move_d {
            self.move_vec.y += self.lin_speed;
        }
        if move_l {
            self.move_vec.x -= self.lin_speed;
        }
        if move_r {
            self.move_vec.x += self.lin_speed;
        }

        let rot = Rotation::new(self.get_rotation());
        self.move_vec = rot * self.move_vec;

        let rot_l = rl.is_key_down(KeyboardKey::KEY_I);
        let rot_r = rl.is_key_down(KeyboardKey::KEY_O);

        self.rot = 0.0;
        if rot_l {
            self.rot -= self.ang_speed;
        }
        if rot_r {
            self.rot += self.ang_speed;
        }
    }
}

impl PhysicsObject for Player {
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
        body.set_linvel(body.linvel() + self.move_vec * delta, true);
        body.set_angvel(body.angvel() + self.rot * delta, true);
        self.game_object.physics_process(delta, body);
    }

    fn update_state(&mut self, body: &RigidBody) {
        self.game_object.update_state(body);
    }
}
