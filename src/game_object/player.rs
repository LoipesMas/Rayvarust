use super::{Drawable, GameObject, PhysicsObject, Processing, Spatial, Sprite};

use crate::{impl_drawable, impl_spatial};

use raylib::prelude::*;

use crate::math::NVector2;

use rapier2d::prelude::*;

pub struct Player {
    game_object: GameObject,
    pub lin_speed: f32,
    pub ang_speed: f32,
    move_vec: NVector2, // add this to lin vel on next phys process
    rot: f32,           // add this to ang vel on next phys process
    zoom: f32,
    pub fuel: f32,
    pub level_completed: bool,
}

#[allow(dead_code)]
impl Player {
    pub fn new(texture: WeakTexture2D) -> Self {
        let mut game_object = GameObject::new();
        game_object.sprite = Some(Sprite::new(texture, true, 0.7));

        Player {
            game_object,
            lin_speed: 70.0,
            ang_speed: 1.0,
            move_vec: NVector2::zeros(),
            rot: 0.0,
            zoom: 0.3,
            fuel: 0.,
            level_completed: false,
        }
    }
    pub fn get_zoom(&self) -> f32 {
        self.zoom
    }
}

impl_spatial!(Player);
impl_drawable!(Player);

impl Processing for Player {
    fn process(&mut self, rl: &mut RaylibHandle, delta: f32) {
        // Movement
        let move_u = rl.is_key_down(KeyboardKey::KEY_W);
        let move_d = rl.is_key_down(KeyboardKey::KEY_S);
        let move_l = rl.is_key_down(KeyboardKey::KEY_A);
        let move_r = rl.is_key_down(KeyboardKey::KEY_D);

        self.move_vec = vector![0., 0.];

        let mut moves_count = 0;
        if move_u {
            self.move_vec.y -= self.lin_speed * 3.0;
            moves_count += 1;
        }
        if move_d {
            self.move_vec.y += self.lin_speed;
            moves_count += 1;
        }
        if move_l {
            self.move_vec.x -= self.lin_speed;
            moves_count += 1;
        }
        if move_r {
            self.move_vec.x += self.lin_speed;
            moves_count += 1;
        }
        if !self.level_completed {
            self.fuel -= delta * 10. * moves_count as f32;
        }

        let rot = Rotation::new(self.get_rotation());
        self.move_vec = rot * self.move_vec;

        // Rotating
        let rot_l = rl.is_key_down(KeyboardKey::KEY_I);
        let rot_r = rl.is_key_down(KeyboardKey::KEY_O);

        self.rot = 0.0;
        if rot_l {
            self.rot -= self.ang_speed;
        }
        if rot_r {
            self.rot += self.ang_speed;
        }

        // Zoom
        let zoom_plus = rl.is_key_down(KeyboardKey::KEY_LEFT_BRACKET);
        let zoom_minus = rl.is_key_down(KeyboardKey::KEY_RIGHT_BRACKET);

        if zoom_plus {
            self.zoom *= 1.0 / (1.0 + delta * 4.0);
        }
        if zoom_minus {
            self.zoom *= 1.0 + delta * 4.0;
        }
        self.zoom = self.zoom.min(3.).max(0.18);
    }
}

impl PhysicsObject for Player {
    fn get_body(&self) -> &RigidBodyHandle {
        self.game_object.get_body()
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
