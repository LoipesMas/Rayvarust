use crate::GameObject;
use crate::Component;
use rapier2d::prelude::*;

pub struct PhysicsBody {
    pub rigid_body: RigidBodyHandle,
    pub collider: ColliderHandle,

}

impl Component for PhysicsBody {
    #[allow(unused_variables)]
    fn process(&mut self, delta: f32, owner: &GameObject) {
    }
}
