use crate::game_object::GameObject;

mod physics_body;

pub trait Component {
    fn process(&mut self, delta: f32, owner: &GameObject);
}
