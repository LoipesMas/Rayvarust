use crate::math::*;
use rapier2d::prelude::*;
use raylib::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::game_object::*;

mod physics_server;
use physics_server::*;

use rand::prelude::*;
use rand_pcg::Pcg64;

/// Color of debug collider
const COLL_COLOR: Color = Color {
    r: 70,
    g: 200,
    b: 70,
    a: 130,
};

const G: f32 = 10.0;

pub struct Game<'a> {
    rl: &'a mut RaylibHandle,
    thread: &'a RaylibThread,
    rng: Pcg64,
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
    player_tex: WeakTexture2D,
    player_score: i32,
    time_since_start: f32,
    completed: bool,
    camera: Camera2D,
    font: Font,
    asteroid_tex: WeakTexture2D,
    gate_tex: WeakTexture2D,
    gate_count: u32,
    next_gate: u32,
    arrow: GameObject,
    arrow_tex: WeakTexture2D,
}

impl<'a> Game<'a> {
    pub fn new(
        rl: &'a mut RaylibHandle,
        thread: &'a RaylibThread,
        window_width: i16,
        window_height: i16,
        seed: u64,
    ) -> Self {
        let draw_fps = true;
        let draw_collisions = false;

        let font = rl
            .load_font_ex(
                thread,
                "resources/fonts/Roboto-Regular.ttf",
                100,
                FontLoadEx::Default(0),
            )
            .expect("Couldn't load font");

        let player_tex = unsafe {
            rl.load_texture(thread, "resources/textures/spaceship.png")
                .expect("Couldn't load spaceship.png")
                .make_weak()
        };

        let asteroid_tex = unsafe {
            rl.load_texture(thread, "resources/textures/asteroid.png")
                .expect("Couldn't load asteroid.png")
                .make_weak()
        };

        let gate_tex = unsafe {
            rl.load_texture(thread, "resources/textures/gate.png")
                .expect("Couldn't load gate.png")
                .make_weak()
        };

        let arrow_tex = unsafe {
            rl.load_texture(thread, "resources/textures/arrow.png")
                .expect("Couldn't load arrow.png")
                .make_weak()
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

        let mut arrow = GameObject::new();
        arrow.sprite = Some(Sprite::new(arrow_tex.clone(), true, 0.5));

        Game {
            rl,
            thread,
            rng: Pcg64::seed_from_u64(seed),
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
            player_tex,
            player_score: 30,
            time_since_start: 0.,
            completed: false,
            camera,
            font,
            asteroid_tex,
            gate_tex,
            gate_count: 0,
            next_gate: 0,
            arrow,
            arrow_tex,
        }
    }

    pub fn unload(&mut self) {
        unsafe {
            self.rl.unload_texture(self.thread, self.player_tex.clone());
            self.rl
                .unload_texture(self.thread, self.asteroid_tex.clone());
            self.rl.unload_texture(self.thread, self.gate_tex.clone());
            self.rl.unload_texture(self.thread, self.arrow_tex.clone());
        }
    }

    pub fn step(&mut self) -> Option<GameAction> {
        let delta = self.rl.get_frame_time();
        if !self.completed {
            self.time_since_start += delta;
        }

        // Update camera center
        if self.rl.is_window_resized() {
            let window_width = self.rl.get_screen_width();
            let window_height = self.rl.get_screen_height();
            self.camera.offset =
                Vector2::new(window_width as f32 / 2.0, window_height as f32 / 2.0);
        }
        // Always center mouse
        self.rl.set_mouse_position(self.camera.offset);

        // Go back to menu
        if self.rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
            return Some(GameAction::Menu);
        }
        // Restart game
        if self.rl.is_key_pressed(KeyboardKey::KEY_R) {
            return Some(GameAction::Restart);
        }

        // For debug
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
            // Only calculate gravity for dynamic objects
            if body.is_dynamic() {
                // Calculate gravity
                let mut gravity_force = vector![0., 0.];
                for planet_v in planets_vector.iter() {
                    let dir = planet_v.0 - body.translation();
                    let dist = dir.norm();
                    if dist > 7777.7 {
                        continue;
                    }
                    gravity_force +=
                        dir.normalize() * G * planet_v.1 / dir.norm_squared().max(0.01);
                }
                // Apply gravity
                body.apply_force(gravity_force * body.mass(), true);
            }
            // Call objects physics process
            object.borrow_mut().physics_process(delta, body);
        }

        // Physics
        self.physics_server
            .step(&mut self.rigid_body_set, &mut self.collider_set);

        let mut contact_events = self
            .physics_server
            .event_handler
            .contact_events
            .lock()
            .unwrap();
        for event in contact_events.drain(..) {
            #[allow(unused_variables)]
            if let ContactEvent::Started(col1, col2) = event {
                if !self.completed {
                    self.player_score -= 10;
                }
            }
        }

        // When player goes through a gate
        if self.physics_server.player_intersected {
            // Get collider
            if let Some(col_h) = self.physics_server.last_intersected {
                // Get body
                let body_h = &self.collider_set[col_h].parent();
                if let Some(body_h) = body_h {
                    // Check if gate number is the one that player should go through
                    let body = &self.rigid_body_set[*body_h];
                    if body.user_data == self.next_gate.into() {
                        // "Select" next gate
                        self.next_gate += 1;
                        self.player_score += 30;
                    }
                }
            }
        }

        self.completed = self.next_gate >= self.gate_count;

        // Update state of all physics objects
        // (This makes their position and rotation the same as their rigidbodies')
        for object in self.phys_objects.iter_mut() {
            let body = &self.rigid_body_set[*object.borrow().get_body()];
            object.borrow_mut().update_state(body);
        }

        // Camera follows player
        if let Some(player_rc) = &self.player_rc {
            self.camera.target = to_rv2(lerp(
                to_nv2(self.camera.target),
                to_nv2(player_rc.borrow().get_position()),
                0.15,
            ));
            self.camera.rotation = -player_rc.borrow().get_rotation() * RAD2DEG as f32;
            // Player controls zoom
            self.camera.zoom = player_rc.borrow().get_zoom();
        }

        // Drawing
        let mut d = self.rl.begin_drawing(self.thread);

        // Camera mode
        {
            let mut mode = d.begin_mode2D(self.camera);
            mode.clear_background(self.bg_color);

            // Render gates first
            for gate in self.gate_objects.iter() {
                use std::cmp::Ordering;

                let mut gate = gate.borrow_mut();

                // Color based on whether the gate is past/current/future
                let color = match gate.gate_num.cmp(&self.next_gate) {
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

            // Draw arrow to next gate
            if !self.completed {
                if let Some(player) = &self.player_rc {
                    let player = player.borrow();
                    let pl_pos = player.get_position();
                    let next_pos = self
                        .gate_objects
                        .get(self.next_gate as usize)
                        .unwrap()
                        .borrow()
                        .get_position();
                    let dir = pl_pos - next_pos;
                    if dir.length() > 256.0 {
                        let angle = dir.angle_to(Vector2::new(-1., 0.));
                        let pos = pl_pos - dir.normalized() * 64.0;
                        self.arrow.set_position(pos);
                        self.arrow.set_rotation(angle);
                        self.arrow.draw(&mut mode);
                    }
                }
            }

            // Draw collisions
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

        // Draw UI
        {
            // Draw fps in top-left corner
            if self.draw_fps {
                d.draw_text_ex(
                    &self.font,
                    &format!("{:.1}", 1.0 / delta),
                    Vector2::new(1825., 0.),
                    50.0,
                    1.0,
                    Color::WHITE,
                );
            }

            let mut line = -1.;

            // Player score
            let score_text = format!("Score: {:}", self.player_score);
            line += 1.0;
            d.draw_text_ex(
                &self.font,
                &score_text,
                Vector2 {
                    x: 0.0,
                    y: 50.0 * line,
                },
                50.0,
                0.0,
                Color::GREEN,
            );

            // Gates
            let gates_text = format!("Gates: {}/{}", self.next_gate, self.gate_count);
            line += 1.0;
            d.draw_text_ex(
                &self.font,
                &gates_text,
                Vector2 {
                    x: 0.0,
                    y: 50.0 * line,
                },
                50.0,
                0.0,
                Color::GREEN,
            );

            // Time
            let time_text = format!("Time: {:.2}", self.time_since_start);
            line += 1.0;
            d.draw_text_ex(
                &self.font,
                &time_text,
                Vector2 {
                    x: 0.0,
                    y: 50.0 * line,
                },
                50.0,
                0.0,
                Color::GREEN,
            );

            // Fuel
            let fuel_text = format!("Fuel: {}", 100.);
            line += 1.0;
            d.draw_text_ex(
                &self.font,
                &fuel_text,
                Vector2 {
                    x: 0.0,
                    y: 50.0 * line,
                },
                50.0,
                0.0,
                Color::GREEN,
            );

            // Restart prompt
            if self.completed {
                let restart_text = "Press R to restart";
                line += 1.0;
                d.draw_text_ex(
                    &self.font,
                    restart_text,
                    Vector2 {
                        x: 0.0,
                        y: 50.0 * line,
                    },
                    50.0,
                    0.0,
                    Color::GREEN,
                );
            }
        }

        None
    }

    /// Runs the game
    pub fn run(&mut self) -> GameAction {
        while !self.rl.window_should_close() {
            let action = self.step();
            if let Some(action) = action {
                return action;
            }
        }
        GameAction::Quit
    }

    /// Spawns player
    pub fn spawn_player(&mut self, position: NVector2) {
        assert!(self.player_rc.is_none(), "Can't spawn second player");
        let mut player = Player::new(self.player_tex.clone());

        let rigid_body = RigidBodyBuilder::new_dynamic()
            .translation(position)
            .can_sleep(false)
            .build();
        let collider = ColliderBuilder::capsule_y(20.0, 20.0)
            .position(Isometry::new(vector![0., 0.0], 0.0))
            .active_events(ActiveEvents::INTERSECTION_EVENTS | ActiveEvents::CONTACT_EVENTS)
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
                asteroid.sprite = Some(Sprite::new(self.asteroid_tex.clone(), true, 0.3));
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

    /// Spawns a planet at given position with given radius
    pub fn spawn_planet(&mut self, position: NVector2, radius: f32, color: Color) {
        let mut planet = Planet::new(to_rv2(position), 0., radius, color);

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

    /// Spawns a gate at given position
    pub fn spawn_gate(&mut self, position: NVector2, rotation: f32) {
        let mut gate = Gate::new(self.gate_tex.clone());
        gate.gate_num = self.gate_count;

        let width = 15.0;
        let height = 115.0;

        let rigid_body = RigidBodyBuilder::new_static()
            .translation(position)
            .can_sleep(false)
            .user_data(self.gate_count.into())
            .rotation(rotation)
            .build();

        let area_collider = ColliderBuilder::cuboid(width * 0.3, height)
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

    /// Spawns a planet with gates around it
    pub fn spawn_planet_with_gates(&mut self, position: NVector2, radius: f32, gate_count: u16) {
        use std::f32::consts::PI;

        assert!(gate_count < 6, "Gate count must be less than 6");

        let hue = self.rng.gen::<f32>() * 250.;
        let sat = self.rng.gen::<f32>() * 0.5 + 0.5;
        let color = Color::color_from_hsv(hue, sat, 0.7);

        self.spawn_planet(position, radius, color);

        let direction = (self.rng.gen::<f32>() - 0.5).signum();

        let start_angle = self.rng.gen::<f32>() * PI;

        let angle_step = 2.0 * PI / (5.0 + self.rng.gen::<f32>() * 2.0);
        for i in 0..gate_count {
            let gate_offset: f32 = radius * (self.rng.gen::<f32>() + 1.2) + 100.;
            let rot = Rotation::new(start_angle + angle_step * direction * i as f32);
            let offset = rot.into_inner() * gate_offset;
            let pos = vector![offset.re, offset.im] + position;
            self.spawn_gate(pos, rot.angle() + PI / 2.0);
        }
    }

    /// Spawns many planets at random positions with gates around them
    pub fn spawn_many_planets_with_gates(&mut self, num_gates: u16) {
        use std::f32::consts::PI;
        let mut planets: Vec<(NVector2, f32)> = Vec::new();

        let radius_range = 300.0..700.0;

        let mut gates_left = num_gates;

        let mut last_position: NVector2 = vector![0., 0.];
        let mut last_radius = 0.;
        while gates_left > 0 {
            let mut position_valid = false;
            let radius = self.rng.gen_range(radius_range.clone());
            let distance = (last_radius + radius) * (3.0 + self.rng.gen::<f32>());
            let mut pos: NVector2 = vector![0., 0.];
            while !position_valid {
                let angle = self.rng.gen::<f32>() * PI * 2.0;
                let rot = Rotation::new(angle);
                let offset = rot.into_inner() * distance;
                pos = vector![offset.re, offset.im] + last_position;

                // Check if planet too close to other planets
                position_valid = true;
                for planet in planets.iter() {
                    let dist = (pos - planet.0).norm();
                    let min_dist = (radius + planet.1) * 2.8;
                    if dist < min_dist {
                        position_valid = false;
                        break;
                    }
                }
            }

            let mut gate_count =
                ((self.rng.gen_range(1..6) + self.rng.gen_range(1..6)) as f32 * 0.5).ceil() as u16;
            gate_count = gate_count.min(gates_left);
            gates_left -= gate_count;
            self.spawn_planet_with_gates(pos, radius, gate_count);
            planets.push((pos, radius));

            last_radius = radius;
            last_position = pos;
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum GameAction {
    Menu,
    Restart,
    Quit,
}
