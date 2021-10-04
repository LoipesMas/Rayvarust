use super::{Drawable, GameObject, PhysicsObject, Processing, Spatial, Sprite};

use crate::physics::*;

use raylib::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

pub struct Player {
    game_object: GameObject,
    pub speed: f32,
}

#[allow(dead_code)]
impl Player {
    pub fn new(texture: Rc<RefCell<Texture2D>>) -> Player {
        let mut game_object = GameObject::new();
        game_object.sprite = Some(Sprite::new(texture, true, 0.7));

        let phys_body = PhysicsBody::new(CollisionShape::Circle(Vector2::zero(), 40.));
        game_object.physics_body = Some(phys_body);

        Player {
            game_object,
            speed: 60.0,
        }
    }
}

impl Drawable for Player {
    fn draw(&self, rl: &mut RaylibMode2D<RaylibDrawHandle>) {
        self.game_object.draw(rl);
    }

    fn get_scale(&self) -> f32 {
        self.game_object.get_scale()
    }

    fn set_scale(&mut self, scale: f32) {
        self.game_object.set_scale(scale);
    }
}

impl Spatial for Player {
    fn get_position(&self) -> Vector2 {
        self.game_object.get_position()
    }
    fn set_position(&mut self, position: Vector2) {
        self.game_object.set_position(position);
    }
    fn translate(&mut self, vector: Vector2) {
        self.set_position(self.get_position() + vector);
    }
}

impl Processing for Player {
    fn process(&mut self, rl: &mut RaylibHandle, delta: f32) {
        let move_u = rl.is_key_down(KeyboardKey::KEY_W);
        let move_d = rl.is_key_down(KeyboardKey::KEY_S);
        let move_l = rl.is_key_down(KeyboardKey::KEY_A);
        let move_r = rl.is_key_down(KeyboardKey::KEY_D);

        let mut move_vec = Vector2::zero();

        if move_u {
            move_vec.y -= self.speed * 2.0;
        }
        if move_d {
            move_vec.y += self.speed;
        }
        if move_l {
            move_vec.x -= self.speed;
        }
        if move_r {
            move_vec.x += self.speed;
        }

        move_vec *= delta;

        self.game_object
            .get_body_mut()
            .add_linear_velocity(move_vec);
    }
}

impl PhysicsObject for Player {
    fn get_body(&self) -> &PhysicsBody {
        self.game_object.get_body()
    }

    fn get_body_mut(&mut self) -> &mut PhysicsBody {
        self.game_object.get_body_mut()
    }

    fn physics_process(&mut self, delta: f32) {
        self.game_object.physics_process(delta);
    }
}
