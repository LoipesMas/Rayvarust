use crate::math::*;
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

const G: f32 = 10.0;

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
    planet_objects: Vec<Rc<RefCell<Planet>>>,
    gate_objects: Vec<Rc<RefCell<Gate>>>,
    player_rc: Option<Rc<RefCell<Player>>>,
    camera: Camera2D,
    font: WeakFont,
    asteroid_tex_ref: Rc<RefCell<WeakTexture2D>>,
    gate_tex_ref: Rc<RefCell<WeakTexture2D>>,
    gate_count: u32,
    next_gate: u32,
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
            .expect("Couldn't load font")
            .make_weak();

        let asteroid_tex_ref = unsafe {
            let asteroid_tex = rl
                .load_texture(&thread, "resources/textures/asteroid.png")
                .expect("Couldn't load asteroid.png")
                .make_weak();
            Rc::new(RefCell::new(asteroid_tex))
        };

        let gate_tex_ref = unsafe {
            let gate_tex = rl
                .load_texture(&thread, "resources/textures/gate.png")
                .expect("Couldn't load gate.png")
                .make_weak();
            Rc::new(RefCell::new(gate_tex))
        };

        let bg_color = color::rcolor(47, 40, 70, 255);

        // Initialize physics
        let rigid_body_set = RigidBodySet::new();
        let collider_set = ColliderSet::new();

        let process_objects: Vec<Rc<RefCell<dyn Processing>>> = Vec::new();
        let draw_objects: Vec<Rc<RefCell<dyn Drawable>>> = Vec::new();
        let phys_objects: Vec<Rc<RefCell<dyn PhysicsObject>>> = Vec::new();
        let planet_objects: Vec<Rc<RefCell<Planet>>> = Vec::new();
        let gate_objects: Vec<Rc<RefCell<Gate>>> = Vec::new();

        let camera = Camera2D {
            offset: Vector2::new(window_width as f32 / 2.0, window_height as f32 / 2.0),
            target: Vector2::new(0., 0.),
            rotation: 0.,
            zoom: 0.66,
        };

        let physics_server = PhysicsServer::new();

        rl.hide_cursor();

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
            planet_objects,
            gate_objects,
            player_rc: None,
            camera,
            font,
            asteroid_tex_ref,
            gate_tex_ref,
            gate_count: 0,
            next_gate: 0,
        }
    }

    pub fn step(&mut self) {
        let delta = self.rl.get_frame_time();

        if self.rl.is_key_pressed(KeyboardKey::KEY_B) {
            self.draw_collisions ^= true;
        }

        // Processing
        for object in &self.process_objects {
            object.borrow_mut().process(&mut self.rl, delta);
        }

        // Calculating gravity forces
        let mut planets_vector: Vec<(NVector2, f32)> = Vec::new();
        for planet in self.planet_objects.iter() {
            let pos = planet.borrow().get_position();
            let mass = planet.borrow().get_mass();
            planets_vector.push((vector![pos.x, pos.y], mass));
        }

        // Pre physics
        for object in self.phys_objects.iter_mut() {
            let body = &mut self.rigid_body_set[*object.borrow().get_body()];
            // Apply gravity
            let mut gravity_force = vector![0., 0.];
            for planet_v in planets_vector.iter() {
                let dir = planet_v.0 - body.translation();
                gravity_force += dir.normalize() * G * planet_v.1 / dir.norm_squared().max(0.01);
            }
            body.apply_force(gravity_force * body.mass(), true);
            object.borrow_mut().physics_process(delta, body);
        }

        // Physics
        self.physics_server
            .step(&mut self.rigid_body_set, &mut self.collider_set);

        if self.physics_server.player_intersected {
            if let Some(col_h) = self.physics_server.last_intersected {
                let body_h = &self.collider_set[col_h].parent();
                if let Some(body_h) = body_h {
                    let body = &self.rigid_body_set[*body_h];
                    if body.user_data == self.next_gate.into() {
                        self.next_gate += 1;
                    }
                }
            }
        }

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
            self.camera.zoom = player_rc.borrow().get_zoom();
        }

        // Drawing
        let mut d = self.rl.begin_drawing(&self.thread);

        {
            let mut mode = d.begin_mode2D(self.camera);
            mode.clear_background(self.bg_color);

            // Render gates first
            for gate in self.gate_objects.iter() {
                use std::cmp::Ordering;

                let mut gate = gate.borrow_mut();
                let color =match gate.gate_num.cmp(&self.next_gate) {
                    Ordering::Less => Color::GRAY,
                    Ordering::Equal => HIGHLIGHT_COLOR,
                    Ordering::Greater => Color::WHITE,
                };
                gate.set_tint(color);
                gate.draw(&mut mode);
            }

            // Rendering objects
            for object in &self.draw_objects {
                object.borrow().draw(&mut mode);
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

        // Draw player score
        d.draw_text_ex(
            &self.font,
            &self.next_gate.to_string(),
            Vector2 { x: 0.0, y: 50.0 },
            50.0,
            1.0,
            Color::GREEN,
        );
    }

    pub fn run(&mut self) {
        while !self.rl.window_should_close() {
            self.step()
        }
    }

    pub fn spawn_player(&mut self, position: NVector2) {
        let player_tex_ref = unsafe {
            let player_tex = self
                .rl
                .load_texture(&self.thread, "resources/textures/spaceship.png")
                .expect("Couldn't load spaceship.png")
                .make_weak();
            Rc::new(RefCell::new(player_tex))
        };

        let mut player = Player::new(player_tex_ref);

        let rigid_body = RigidBodyBuilder::new_dynamic()
            .translation(position)
            .can_sleep(false)
            .build();
        let collider = ColliderBuilder::capsule_y(20.0, 20.0)
            .position(Isometry::new(vector![0., 0.0], 0.0))
            .active_events(ActiveEvents::INTERSECTION_EVENTS)
            .build();

        player.update_state(&rigid_body);

        let player_body_handle = self.rigid_body_set.insert(rigid_body);
        let player_col_handle = self.collider_set.insert_with_parent(
            collider,
            player_body_handle,
            &mut self.rigid_body_set,
        );
        self.physics_server.player_collider_handle = Some(player_col_handle);

        player.set_body(player_body_handle);

        let player_rc = Rc::new(RefCell::new(player));
        self.process_objects.push(player_rc.clone());
        self.draw_objects.push(player_rc.clone());
        self.phys_objects.push(player_rc.clone());
        self.player_rc = Some(player_rc);
    }

    /// Spawns 100 asteroids
    /// (For testing)
    // TODO: make seperate function for spawning an asteroid
    pub fn spawn_asteroids(&mut self) {
        let center = vector![63. * 4.5, 50. * 4.5];

        // Spawn 100 asteroids
        for i in 0..10 {
            for j in 0..10 {
                let mut asteroid = GameObject::new();
                asteroid.sprite = Some(Sprite::new(self.asteroid_tex_ref.clone(), true, 0.3));
                let pos = vector![60. * i as f32, 50. * j as f32];

                let mut rigid_body = RigidBodyBuilder::new_dynamic()
                    .translation(pos)
                    .can_sleep(false)
                    .build();
                let collider = ColliderBuilder::capsule_y(0.0, 13.0)
                    .restitution(0.8)
                    .density(0.5)
                    .build();

                let mut vel = center - pos;
                vel.normalize_mut();
                vel *= 30.;
                rigid_body.set_linvel(vel, true);

                let rigid_body_handle = self.rigid_body_set.insert(rigid_body);
                self.collider_set.insert_with_parent(
                    collider,
                    rigid_body_handle,
                    &mut self.rigid_body_set,
                );
                asteroid.set_body(rigid_body_handle);

                let asteroid_rc = Rc::new(RefCell::new(asteroid));
                self.process_objects.push(asteroid_rc.clone());
                self.draw_objects.push(asteroid_rc.clone());
                self.phys_objects.push(asteroid_rc);
            }
        }
    }

    /// Spawn a planet at given position with given radius
    pub fn spawn_planet(&mut self, position: NVector2, radius: f32) {
        let mut planet = Planet::new(to_rv2(position), 0., radius);

        let rigid_body = RigidBodyBuilder::new_static()
            .translation(position)
            .can_sleep(false)
            .build();
        let collider = ColliderBuilder::ball(radius).density(5.0).build();

        let rigid_body_handle = self.rigid_body_set.insert(rigid_body);
        self.collider_set
            .insert_with_parent(collider, rigid_body_handle, &mut self.rigid_body_set);
        planet.set_body(rigid_body_handle);

        let planet_rc = Rc::new(RefCell::new(planet));
        self.draw_objects.push(planet_rc.clone());
        self.phys_objects.push(planet_rc.clone());
        self.planet_objects.push(planet_rc);
    }

    /// Spawn a gate at given position
    pub fn spawn_gate(&mut self, position: NVector2) {
        let mut gate = Gate::new(self.gate_tex_ref.clone());
        gate.gate_num = self.gate_count;

        let width = 15.0;
        let height = 115.0;

        let rigid_body = RigidBodyBuilder::new_static()
            .translation(position)
            .can_sleep(false)
            .user_data(self.gate_count.into())
            .build();
        let area_collider = ColliderBuilder::cuboid(width * 0.5, height)
            .sensor(true)
            .build();
        let gate_collider_1 = ColliderBuilder::ball(width)
            .translation(vector![0., height])
            .build();
        let gate_collider_2 = ColliderBuilder::ball(width)
            .translation(vector![0., -(height)])
            .build();

        let rigid_body_handle = self.rigid_body_set.insert(rigid_body);
        self.collider_set.insert_with_parent(
            area_collider,
            rigid_body_handle,
            &mut self.rigid_body_set,
        );

        self.collider_set.insert_with_parent(
            gate_collider_1,
            rigid_body_handle,
            &mut self.rigid_body_set,
        );
        self.collider_set.insert_with_parent(
            gate_collider_2,
            rigid_body_handle,
            &mut self.rigid_body_set,
        );

        gate.set_body(rigid_body_handle);

        let gate_rc = Rc::new(RefCell::new(gate));
        self.phys_objects.push(gate_rc.clone());
        self.gate_objects.push(gate_rc);

        self.gate_count += 1;
    }
}
