use crate::math::{to_rv2, lerp, to_nv2};
use rapier2d::prelude::*;
use raylib::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::game_object::*;

mod physics_server;
use physics_server::*;

/// Color of debug collider
const COLL_COLOR: Color = Color {
    r: 70,
    g: 200,
    b: 70,
    a: 130,
};

pub struct Game {
    rl: RaylibHandle,
    thread: RaylibThread,
    draw_fps: bool,
    draw_collisions: bool,
    bg_color: Color,
    physics_server: PhysicsServer,
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    process_objects: Vec<Rc<RefCell<dyn Processing>>>,
    draw_objects: Vec<Rc<RefCell<dyn Drawable>>>,
    phys_objects: Vec<Rc<RefCell<dyn PhysicsObject>>>,
    player_rc: Option<Rc<RefCell<Player>>>,
    camera: Camera2D,
    font: Font, 
    asteroid_tex_ref: Rc<RefCell<Texture2D>>,
}

impl Game {
    pub fn new() -> Self {
        let window_width: i16 = 960 * 2;
        let window_height: i16 = 540 * 2;
        let draw_fps = true;
        let draw_collisions = false;

        let (mut rl, thread) = raylib::init()
            .size(window_width.into(), window_height.into())
            .title("Rayvarust")
            .fullscreen()
            .vsync()
            .build();

        let font = rl
            .load_font(&thread, "resources/fonts/RobotoMono-Regular.ttf")
            .expect("Couldn't load font");

        let asteroid_tex = rl
            .load_texture(&thread, "resources/textures/asteroid.png")
            .expect("Couldn't load astronaut.png");
        let asteroid_tex_ref = Rc::new(RefCell::new(asteroid_tex));

        let bg_color = color::rcolor(47, 40, 70, 255);

        // Initialize physics
        let rigid_body_set = RigidBodySet::new();
        let collider_set = ColliderSet::new();

        let process_objects: Vec<Rc<RefCell<dyn Processing>>> = Vec::new();
        let draw_objects: Vec<Rc<RefCell<dyn Drawable>>> = Vec::new();
        let phys_objects: Vec<Rc<RefCell<dyn PhysicsObject>>> = Vec::new();

        let camera = Camera2D {
            offset: Vector2::new(window_width as f32 / 2.0, window_height as f32 / 2.0),
            target: Vector2::new(0., 0.),
            rotation: 0.,
            zoom: 0.66,
        };

        let physics_server = PhysicsServer::new();

        Game {
            rl,
            thread,
            draw_fps,
            draw_collisions,
            bg_color,
            physics_server,
            rigid_body_set,
            collider_set,
            process_objects,
            draw_objects,
            phys_objects,
            player_rc: None,
            camera,
            font,
            asteroid_tex_ref
        }
        
    }

    pub fn step(&mut self) {
        let delta = self.rl.get_frame_time();

        // Processing
        for object in &self.process_objects {
            object.borrow_mut().process(&mut self.rl, delta);
        }

        for object in self.phys_objects.iter_mut() {
            let body = &mut self.rigid_body_set[*object.borrow().get_body()];
            object.borrow_mut().physics_process(delta, body);
        }

        // Physics
        self.physics_server.step(&mut self.rigid_body_set, &mut self.collider_set);


        for object in self.phys_objects.iter_mut() {
            let body = &self.rigid_body_set[*object.borrow().get_body()];
            object.borrow_mut().update_state(body);
        }

        if let Some(player_rc) = &self.player_rc {
            self.camera.target = to_rv2(lerp(
                to_nv2(self.camera.target),
                to_nv2(player_rc.borrow().get_position()),
                0.17,
            ));
            self.camera.rotation = -player_rc.borrow().get_rotation() * RAD2DEG as f32;
        }

        let mut d = self.rl.begin_drawing(&self.thread);

        {
            let mut mode = d.begin_mode2D(self.camera);
            mode.clear_background(self.bg_color);

            // Rendering objects
            for object in &self.draw_objects {
                object.borrow_mut().draw(&mut mode);
            }

            // Draw collision
            if self.draw_collisions {
                for object in self.phys_objects.iter() {
                    let body = &self.rigid_body_set[*object.borrow().get_body()];
                    for collider in body.colliders() {
                        let collider = &self.collider_set[*collider];
                        let aabb = collider.shape().compute_local_aabb();
                        let h_width = aabb.half_extents()[0];
                        let h_height = aabb.half_extents()[1];
                        let rec = Rectangle::new(
                            collider.translation().x,
                            collider.translation().y,
                            h_width * 2.0,
                            h_height * 2.0,
                        );
                        let origin = Vector2::new(h_width, h_height);
                        mode.draw_rectangle_pro(
                            rec,
                            origin,
                            RAD2DEG as f32 * body.rotation().angle(),
                            COLL_COLOR,
                        );
                    }
                }
            }
        }

        // Draw fps
        if self.draw_fps {
            d.draw_text_ex(
                &self.font,
                &(1. / delta).to_string(),
                Vector2::zero(),
                50.0,
                1.0,
                Color::WHITE,
            );
        }
    }

    pub fn run(&mut self) {
        while !self.rl.window_should_close() {
            self.step()
        }
    }

    pub fn spawn_player(&mut self, ) {
        let player_tex = self.rl
            .load_texture(&self.thread, "resources/textures/spaceship.png")
            .expect("Couldn't load spaceship.png");
        let player_tex_ref = Rc::new(RefCell::new(player_tex));

        let mut player = Player::new(Rc::clone(&player_tex_ref));

        let rigid_body = RigidBodyBuilder::new_dynamic()
            .translation(vector![0.0, 10.0])
            .build();
        let collider = ColliderBuilder::capsule_y(20.0, 20.0)
            .position(Isometry::new(vector![0., 0.0], 0.0))
            .density(2.0)
            .build();
        let player_body_handle = self.rigid_body_set.insert(rigid_body);
        self.collider_set.insert_with_parent(collider, player_body_handle, &mut self.rigid_body_set);
        player.set_body(player_body_handle);

        let player_rc = Rc::new(RefCell::new(player));
        self.process_objects.push(player_rc.clone());
        self.draw_objects.push(player_rc.clone());
        self.phys_objects.push(player_rc.clone());
        self.player_rc = Some(player_rc);
    }

    /// Spawns 100 asteroids
    /// (For testing)
    pub fn spawn_asteroids(&mut self) {
        let center = vector![63. * 4.5, 50. * 4.5];

        // Spawn 100 asteroids
        for i in 0..10 {
            for j in 0..10 {
                let mut asteroid = GameObject::new();
                asteroid.sprite = Some(Sprite::new(Rc::clone(&self.asteroid_tex_ref), true, 0.3));
                let pos = vector![60. * i as f32, 50. * j as f32];

                let mut rigid_body = RigidBodyBuilder::new_dynamic().translation(pos).build();
                let collider = ColliderBuilder::capsule_y(0.0, 13.0).build();

                let mut vel = center - pos;
                vel.normalize_mut();
                vel *= 30.;
                rigid_body.set_linvel(vel, true);

                let rigid_body_handle = self.rigid_body_set.insert(rigid_body);
                self.collider_set.insert_with_parent(collider, rigid_body_handle, &mut self.rigid_body_set);
                asteroid.set_body(rigid_body_handle);

                let asteroid_rc = Rc::new(RefCell::new(asteroid));
                self.process_objects.push(asteroid_rc.clone());
                self.draw_objects.push(asteroid_rc.clone());
                self.phys_objects.push(asteroid_rc);
            }
        }
    }
}
