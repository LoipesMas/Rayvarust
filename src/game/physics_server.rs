use rapier2d::prelude::*;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use crate::math::NVector2;

pub struct PhysicsServer {
    gravity: NVector2,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    joint_set: JointSet,
    ccd_solver: CCDSolver,
    physics_hooks: (),
    pub event_handler: MyEventHandler,
    pub player_collider_handle: Option<ColliderHandle>,
    pub player_intersected: bool,
    pub last_intersected: Option<ColliderHandle>,
}

impl PhysicsServer {
    pub fn new() -> Self {
        /* Create other structures necessary for the simulation. */
        Self {
            gravity: vector![0.0, 0.0],
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            joint_set: JointSet::new(),
            ccd_solver: CCDSolver::new(),
            physics_hooks: (),
            event_handler: MyEventHandler::new(),
            player_collider_handle: None,
            player_intersected: false,
            last_intersected: None,
        }
    }

    pub fn step(&mut self, rigid_body_set: &mut RigidBodySet, collider_set: &mut ColliderSet) {
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            rigid_body_set,
            collider_set,
            &mut self.joint_set,
            &mut self.ccd_solver,
            &self.physics_hooks,
            &self.event_handler,
        );

        self.player_intersected = false;
        if self
            .event_handler
            .player_intersected
            .load(Ordering::Acquire)
        {
            let col1 = (*self.event_handler.collider1.lock().unwrap()).unwrap();
            let col2 = (*self.event_handler.collider2.lock().unwrap()).unwrap();
            if let Some(p_col_handle) = self.player_collider_handle {
                let gate_col = if col1 == p_col_handle {
                    Some(col2)
                } else {
                    Some(col1)
                };
                self.last_intersected = gate_col;
            }
            self.event_handler
                .player_intersected
                .store(false, Ordering::Relaxed);
            self.player_intersected = true;
        }
    }
}

pub struct MyEventHandler {
    pub player_intersected: AtomicBool,
    pub collider1: Mutex<Option<ColliderHandle>>,
    pub collider2: Mutex<Option<ColliderHandle>>,
    pub contact_events: Mutex<Vec<ContactEvent>>,
}

impl MyEventHandler {
    pub fn new() -> Self {
        MyEventHandler {
            player_intersected: AtomicBool::new(false),
            collider1: Mutex::new(None),
            collider2: Mutex::new(None),
            contact_events: Mutex::new(Vec::new()),
        }
    }
}

impl EventHandler for MyEventHandler {
    fn handle_intersection_event(&self, event: IntersectionEvent) {
        let IntersectionEvent {
            collider1,
            collider2,
            intersecting,
        } = event;
        if intersecting {
            return;
        }
        self.player_intersected.store(true, Ordering::Relaxed);
        let mut guard = self.collider1.lock().unwrap();
        *guard = Some(collider1);
        let mut guard = self.collider2.lock().unwrap();
        *guard = Some(collider2);
        drop(guard);
    }

    #[allow(unused_variables)]
    fn handle_contact_event(&self, event: ContactEvent, contact_pair: &ContactPair) {
        let mut guard = self.contact_events.lock().unwrap();
        (*guard).push(event);
    }
}
