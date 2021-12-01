use crate::math::{lerp, to_nv2, to_rv2, NVector2};
use crate::SHIP_NAMES;
use rapier2d::prelude::*;
use raylib::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::game_object::*;

mod physics_server;
use physics_server::*;

use rand::prelude::*;
use rand_pcg::Pcg64;

use std::f32::consts::PI;

use std::collections::HashMap;

#[macro_export]
macro_rules! DrawHandle {
    () =>  { RaylibShaderMode<RaylibMode2D<RaylibTextureMode<RaylibDrawHandle>>> }
}

/// Color of debug collider
const COLL_COLOR: Color = Color {
    r: 70,
    g: 200,
    b: 70,
    a: 130,
};

const G: f32 = 10.0;

const RENDER_DISTANCE: f32 = 12000i32.pow(2) as f32;

pub struct Game<'a> {
    rl: &'a mut RaylibHandle,
    thread: &'a RaylibThread,
    audio: &'a mut RaylibAudio,
    rng: Pcg64,
    fuel_mode: bool,
    draw_fps: bool,
    draw_collisions: bool,
    blur: bool,
    bg_color: Color,
    physics_server: PhysicsServer,
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    process_objects: HashMap<u128, Rc<RefCell<dyn Processing>>>,
    draw_objects: HashMap<u128, Rc<RefCell<dyn Drawable>>>,
    phys_objects: HashMap<u128, Rc<RefCell<dyn PhysicsObject>>>,
    planet_objects: HashMap<u128, Rc<RefCell<Planet>>>,
    asteroid_colliders: HashMap<ColliderHandle, u128>,
    gate_objects: Vec<Rc<RefCell<Gate>>>,
    player_rc: Option<Rc<RefCell<Player>>>,
    player_tex: WeakTexture2D,
    exhaust_tex: WeakTexture2D,
    player_score: i32,
    time_since_start: f32,
    asteroid_spawn_timer: f32,
    completed: bool,
    camera: Camera2D,
    font: Font,
    asteroid_tex: WeakTexture2D,
    gate_tex: WeakTexture2D,
    gate_off_tex: WeakTexture2D,
    gate_darker_tex: WeakTexture2D,
    gate_count: u32,
    next_gate: u32,
    arrow: GameObject,
    arrow_tex: WeakTexture2D,
    planet_shader: Shader,
    blur_shader: Shader,
    def_shader: Shader,
    ren_tex: RenderTexture2D,
    paused: bool,
    impact_sound: Sound,
    thruster_sound: Music,
    thruster_sound2: Music,
    thruster_volume: f32,
    air_sound: Music,
    air_volume: f32,
}

impl<'a> Game<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rl: &'a mut RaylibHandle,
        thread: &'a RaylibThread,
        audio: &'a mut RaylibAudio,
        window_width: i16,
        window_height: i16,
        seed: u64,
        fuel_mode: bool,
        selected_ship: usize,
    ) -> Self {
        let draw_fps = true;
        let draw_collisions = false;
        let blur = true;

        let font = rl
            .load_font_ex(
                thread,
                "resources/fonts/Roboto-Regular.ttf",
                100,
                FontLoadEx::Default(0),
            )
            .unwrap();

        let ship_name = SHIP_NAMES[selected_ship];
        let player_tex = unsafe {
            rl.load_texture(
                thread,
                &("resources/textures/ships/".to_owned() + ship_name + ".png"),
            )
            .unwrap()
            .make_weak()
        };

        let exhaust_tex = unsafe {
            rl.load_texture(thread, "resources/textures/exhaust.png")
                .unwrap()
                .make_weak()
        };

        let asteroid_tex = unsafe {
            rl.load_texture(thread, "resources/textures/asteroid.png")
                .unwrap()
                .make_weak()
        };

        let gate_tex = unsafe {
            rl.load_texture(thread, "resources/textures/gate.png")
                .unwrap()
                .make_weak()
        };

        let gate_off_tex = unsafe {
            rl.load_texture(thread, "resources/textures/gate_off.png")
                .unwrap()
                .make_weak()
        };

        let gate_darker_tex = unsafe {
            rl.load_texture(thread, "resources/textures/gate_darker.png")
                .unwrap()
                .make_weak()
        };

        let arrow_tex = unsafe {
            rl.load_texture(thread, "resources/textures/arrow.png")
                .unwrap()
                .make_weak()
        };

        let planet_shader = rl
            .load_shader(thread, None, Some("resources/shaders/planet.fs"))
            .unwrap();

        let mut blur_shader = rl
            .load_shader(thread, None, Some("resources/shaders/blur.fs"))
            .unwrap();

        // Update blur shader uniforms
        {
            let loc_w = blur_shader.get_shader_location("renderWidth");
            let loc_h = blur_shader.get_shader_location("renderHeight");
            blur_shader.set_shader_value(loc_w, window_width as f32 * 2.0);
            blur_shader.set_shader_value(loc_h, window_height as f32 * 2.0);
        }

        let def_shader = rl.load_shader(thread, None, None).unwrap();

        let bg_color = color::rcolor(47, 40, 70, 255);

        // Initialize physics
        let rigid_body_set = RigidBodySet::new();
        let collider_set = ColliderSet::new();

        let process_objects: HashMap<u128, Rc<RefCell<dyn Processing>>> = HashMap::new();
        let draw_objects: HashMap<u128, Rc<RefCell<dyn Drawable>>> = HashMap::new();
        let phys_objects: HashMap<u128, Rc<RefCell<dyn PhysicsObject>>> = HashMap::new();
        let planet_objects: HashMap<u128, Rc<RefCell<Planet>>> = HashMap::new();
        let gate_objects: Vec<Rc<RefCell<Gate>>> = Vec::new();
        let asteroid_colliders: HashMap<ColliderHandle, u128> = HashMap::new();

        let camera = Camera2D {
            offset: rvec2(window_width as f32 / 2.0, window_height as f32 / 2.0) * 2.0,
            target: rvec2(0., 0.),
            rotation: 0.,
            zoom: 0.66,
        };

        let physics_server = PhysicsServer::new();

        rl.hide_cursor();

        let mut arrow = GameObject::new();
        arrow.sprite = Some(Sprite::new(arrow_tex.clone(), true, 0.5));

        let impact_sound = Sound::load_sound("resources/sound/spaceship_impact.wav").unwrap();

        let mut thruster_sound = Music::load_music_stream(thread, "resources/sound/thruster1.wav").unwrap();
        thruster_sound.looping = true;
        audio.play_music_stream(&mut thruster_sound);

        let mut thruster_sound2 = Music::load_music_stream(thread, "resources/sound/thruster2.wav").unwrap();
        thruster_sound2.looping = true;
        audio.play_music_stream(&mut thruster_sound2);

        let mut air_sound = Music::load_music_stream(thread, "resources/sound/air_release.wav").unwrap();
        air_sound.looping = true;
        audio.play_music_stream(&mut air_sound);

        let ren_tex = rl
            .load_render_texture(thread, window_width as u32 * 2, window_height as u32 * 2)
            .unwrap();

        Game {
            rl,
            thread,
            audio,
            rng: Pcg64::seed_from_u64(seed),
            fuel_mode,
            draw_fps,
            draw_collisions,
            blur,
            bg_color,
            physics_server,
            rigid_body_set,
            collider_set,
            process_objects,
            draw_objects,
            phys_objects,
            planet_objects,
            gate_objects,
            asteroid_colliders,
            player_rc: None,
            player_tex,
            exhaust_tex,
            player_score: 30,
            time_since_start: 0.,
            asteroid_spawn_timer: 0.,
            completed: false,
            camera,
            font,
            asteroid_tex,
            gate_tex,
            gate_off_tex,
            gate_darker_tex,
            gate_count: 0,
            next_gate: 0,
            arrow,
            arrow_tex,
            planet_shader,
            blur_shader,
            def_shader,
            ren_tex,
            paused: false,
            impact_sound,
            thruster_sound,
            thruster_sound2,
            thruster_volume: 0.0,
            air_sound,
            air_volume: 0.0,
        }
    }

    pub fn remove_by_uuid(&mut self, uuid: &u128) {
        self.process_objects.remove(uuid);
        self.draw_objects.remove(uuid);
        self.phys_objects.remove(uuid);
        self.planet_objects.remove(uuid);
    }

    pub fn remove_rigidbody(&mut self, rigid_body: RigidBodyHandle) {
        self.rigid_body_set.remove(
            rigid_body,
            &mut self.physics_server.island_manager,
            &mut self.collider_set,
            &mut self.physics_server.joint_set,
        );
    }

    pub fn remove_asteroid(&mut self, uuid: &u128, col: &ColliderHandle) {
        let asteroid_body = *self.phys_objects.get(uuid).unwrap().borrow().get_body();
        let scale = self.draw_objects.get(uuid).unwrap().borrow().get_scale();
        let rigid_body = self.rigid_body_set.get(asteroid_body).unwrap();
        let pos = *rigid_body.translation();
        let vel = *rigid_body.linvel();
        let dir = vel.normalize();
        if scale > 0.3 {
            // Spawn more smaller asteroids
            for _ in 0..self.rng.gen_range(2..4) {
                // Random velocities
                let speed = vel.norm() * self.rng.gen_range(0.6..1.3);
                let rot = Rotation::new(PI * self.rng.gen_range(0.5..1.5));
                let linvel = (rot * dir) * speed;
                let angvel = self.rng.gen_range(-10.0..10.0);
                let new_scale = scale * self.rng.gen_range(0.3..0.8);

                self.spawn_asteroid(pos - dir, new_scale, RigidBodyVelocity { linvel, angvel });
            }
        }
        self.remove_rigidbody(asteroid_body);
        self.remove_by_uuid(uuid);
        self.asteroid_colliders.remove(col);
    }

    pub fn unload(&mut self) {
        unsafe {
            self.rl.unload_texture(self.thread, self.player_tex.clone());
            self.rl
                .unload_texture(self.thread, self.exhaust_tex.clone());
            self.rl
                .unload_texture(self.thread, self.asteroid_tex.clone());
            self.rl.unload_texture(self.thread, self.gate_tex.clone());
            self.rl
                .unload_texture(self.thread, self.gate_off_tex.clone());
            self.rl
                .unload_texture(self.thread, self.gate_darker_tex.clone());
            self.rl.unload_texture(self.thread, self.arrow_tex.clone());
        }
    }

    pub fn step(&mut self) -> Option<GameAction> {
        let delta = self.rl.get_frame_time();

        if !self.paused {

            // Thruster audio
            self.audio.update_music_stream(&mut self.thruster_sound);
            self.audio.update_music_stream(&mut self.thruster_sound2);
            if self.rl.is_key_down(KeyboardKey::KEY_W) {
                self.thruster_volume *= 1.0 + (delta * 6.0);
            }
            else {
                self.thruster_volume *= 1.0 - (delta * 4.0);
            }
            self.thruster_volume = self.thruster_volume.clamp(0.1, 0.9);
            self.audio.set_music_volume(&mut self.thruster_sound, self.thruster_volume);
            self.audio.set_music_volume(&mut self.thruster_sound2, self.thruster_volume);

            // Air release audio
            self.audio.update_music_stream(&mut self.air_sound);
            if self.rl.is_key_down(KeyboardKey::KEY_S) ||
                self.rl.is_key_down(KeyboardKey::KEY_A) ||
                self.rl.is_key_down(KeyboardKey::KEY_D)
            {
                self.air_volume *= 1.0 + (delta * 6.0);
            }
            else {
                self.air_volume *= 1.0 - (delta * 4.0);
            }
            self.air_volume = self.air_volume.clamp(0.1, 0.9);
            self.audio.set_music_volume(&mut self.air_sound, self.air_volume * 0.05);

            // Tick timers
            if !self.completed {
                self.time_since_start += delta;
            }
            self.asteroid_spawn_timer += delta;
        }

        // Update camera center
        if self.rl.is_window_resized() {
            let window_width = self.rl.get_screen_width();
            let window_height = self.rl.get_screen_height();
            self.camera.offset = rvec2(window_width as f32 / 2.0, window_height as f32 / 2.0) * 2.0;

            // Update blur shader uniforms
            {
                let loc_w = self.blur_shader.get_shader_location("renderWidth");
                let loc_h = self.blur_shader.get_shader_location("renderHeight");
                self.blur_shader
                    .set_shader_value(loc_w, window_width as f32 * 2.0);
                self.blur_shader
                    .set_shader_value(loc_h, window_height as f32 * 2.0);
            }
        }

        let window_width = self.camera.offset.x * 2.0;
        let window_height = self.camera.offset.y * 2.0;
        let camera_diag =
            ((window_width * window_width + window_height * window_height) as f32).sqrt();
        let camera_diag_world = camera_diag / self.camera.zoom;
        let view_r = camera_diag_world * 0.5;

        // Spawning asteroids around the player
        if self.asteroid_spawn_timer > 0.4 && self.asteroid_colliders.len() < 200 {
            let r = view_r * (1.0 + self.rng.gen::<f32>());
            let offset = Rotation::new(self.rng.gen::<f32>() * 2. * PI) * vector![0., 1.] * r;
            let pos = to_nv2(self.camera.target) + offset;
            let linvel = self.rng.gen_range(30.0..300.0)
                * vector![
                    self.rng.gen_range(-1.0..1.0f32),
                    self.rng.gen_range(-1.0..1.0f32)
                ]
                .normalize();
            let angvel = self.rng.gen_range(-10.0..10.0);
            let scale = self.rng.gen_range(0.2..0.6);
            self.spawn_asteroid(pos, scale, RigidBodyVelocity { linvel, angvel });
            self.asteroid_spawn_timer = 0.;
        }

        // Always center mouse
        self.rl.set_mouse_position(self.camera.offset / 2.0);

        // Pause game
        if self.rl.is_key_pressed(KeyboardKey::KEY_TAB) {
            self.paused ^= true;
        }

        // Go back to menu
        if self.rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
            return Some(GameAction::Menu);
        }
        // Restart game
        if self.rl.is_key_pressed(KeyboardKey::KEY_R) {
            return Some(GameAction::Restart);
        }
        // Restart game with new seed
        if self.rl.is_key_pressed(KeyboardKey::KEY_N) {
            return Some(GameAction::NewSeed);
        }

        // Toggle upscaling
        if self.rl.is_key_pressed(KeyboardKey::KEY_B) {
            self.blur ^= true;
        }

        // For debug
        if self.rl.is_key_pressed(KeyboardKey::KEY_C) {
            self.draw_collisions ^= true;
        }

        if !self.paused {
            // Processing
            for object in self.process_objects.values() {
                object.borrow_mut().process(&mut self.rl, delta);
            }

            // Calculating gravity forces
            let mut planets_vector: Vec<(NVector2, f32)> = Vec::new();
            for planet in self.planet_objects.values() {
                let pos = planet.borrow().get_position();
                let mass = planet.borrow().get_mass();
                planets_vector.push((vector![pos.x, pos.y], mass));
            }

            // Pre physics
            for object in self.phys_objects.values_mut() {
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

            let mut contact_events_guard = self
                .physics_server
                .event_handler
                .contact_events
                .lock()
                .unwrap();

            let mut contact_events = contact_events_guard.clone();
            contact_events_guard.clear();
            drop(contact_events_guard);

            for event in contact_events.drain(..) {
                if let ContactEvent::Started(col1, col2) = event {
                    if let Some(pch) = self.physics_server.player_collider_handle {
                        // One of them is the player
                        if col1 == pch || col2 == pch {
                            let bh1 = self.collider_set.get(col1).unwrap().parent().unwrap();
                            let bh2 = self.collider_set.get(col2).unwrap().parent().unwrap();
                            let b1 = self.rigid_body_set.get(bh1).unwrap();
                            let b2 = self.rigid_body_set.get(bh2).unwrap();
                            let vel_dif_mag = (b1.linvel() - b2.linvel()).norm();
                            if vel_dif_mag > 150.0 {
                                self.audio.play_sound_multi(&self.impact_sound);
                            }
                            if !self.completed {
                                self.player_score -= 10;
                            }
                        }
                        // None of them is the player
                        else {
                            // Try to get uuids of asteroids
                            let asteroid1_uuid = self.asteroid_colliders.get(&col1).cloned();
                            let asteroid2_uuid = self.asteroid_colliders.get(&col2).cloned();
                            // Ignore collisions between asteroids
                            if asteroid1_uuid.is_some() && asteroid2_uuid.is_some() {
                                continue;
                            }
                            // Destroy asteroids
                            if let Some(asteroid1_uuid) = asteroid1_uuid {
                                self.remove_asteroid(&asteroid1_uuid, &col1);
                            }
                            if let Some(asteroid2_uuid) = asteroid2_uuid {
                                self.remove_asteroid(&asteroid2_uuid, &col2);
                            }
                        }
                    }
                }
            }

            // When player goes through a gate
            if !self.completed && self.physics_server.player_intersected {
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

            if self.player_score < 0 {
                if let Some(player) = &self.player_rc {
                    let mut player = player.borrow_mut();
                    player.failed = true;
                    self.completed = true;
                }
            }

            self.completed |= self.next_gate >= self.gate_count;

            // Update state of all physics objects
            // (This makes their position and rotation the same as their rigidbodies')
            for object in self.phys_objects.values_mut() {
                let body = &self.rigid_body_set[*object.borrow().get_body()];
                object.borrow_mut().update_state(body);
            }

            // Camera
            if let Some(player_rc) = &self.player_rc {
                // Camera follows player
                self.camera.target = to_rv2(lerp(
                    to_nv2(self.camera.target),
                    to_nv2(player_rc.borrow().get_position()),
                    0.15,
                ));
                self.camera.rotation = -player_rc.borrow().get_rotation() * RAD2DEG as f32;
                // Player controls zoom
                self.camera.zoom = player_rc.borrow().get_zoom();
            }
        }

        let mut d = self.rl.begin_drawing(self.thread);
        d.clear_background(self.bg_color);

        // Drawing to texture
        {
            let mut d = d.begin_texture_mode(self.thread, &mut self.ren_tex);
            d.clear_background(self.bg_color);

            // Camera mode
            {
                let mut mode1 = d.begin_mode2D(self.camera);

                let color_a_loc = self.planet_shader.get_shader_location("colorA");
                let color_b_loc = self.planet_shader.get_shader_location("colorB");
                for planet in self.planet_objects.values_mut() {
                    let planet = planet.borrow();
                    let dist = (planet.get_position() - self.camera.target).length_sqr();
                    if dist > RENDER_DISTANCE {
                        continue;
                    }
                    self.planet_shader
                        .set_shader_value(color_a_loc, planet.color_a.color_normalize());
                    self.planet_shader
                        .set_shader_value(color_b_loc, planet.color_b.color_normalize());
                    let mut mode = mode1.begin_shader_mode(&self.planet_shader);
                    planet.draw(&mut mode);
                }

                let mut mode = mode1.begin_shader_mode(&self.def_shader);

                // Rendering objects
                for object in self.draw_objects.values() {
                    let object = object.borrow();
                    let dist = (object.get_transform().position - self.camera.target).length_sqr();
                    if dist > RENDER_DISTANCE {
                        continue;
                    }

                    object.draw(&mut mode);
                }

                // Render gates last
                for gate in self.gate_objects.iter_mut() {
                    use std::cmp::Ordering;

                    let mut gate = gate.borrow_mut();

                    let dist = (gate.get_position() - self.camera.target).length_sqr();
                    if dist > RENDER_DISTANCE {
                        continue;
                    }

                    // Gates already passed are off
                    match gate.gate_num.cmp(&self.next_gate) {
                        Ordering::Less => gate.set_state(true, false),
                        Ordering::Equal => gate.set_state(false, true),
                        Ordering::Greater => gate.set_state(false, false),
                    };

                    gate.draw(&mut mode);
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
                            let angle = dir.angle_to(rvec2(-1., 0.));
                            let pos = pl_pos - dir.normalized() * 64.0;
                            self.arrow.set_position(pos);
                            self.arrow.set_rotation(angle);
                            self.arrow.draw(&mut mode);
                        }
                    }
                }

                // Draw collisions
                if self.draw_collisions {
                    for object in self.phys_objects.values() {
                        let body = &self.rigid_body_set[*object.borrow().get_body()];
                        for collider in body.colliders() {
                            let collider = &self.collider_set[*collider];
                            let aabb = collider.shape().compute_local_aabb();
                            let h_width = aabb.half_extents()[0];
                            let h_height = aabb.half_extents()[1];
                            let rec = rrect(
                                collider.translation().x,
                                collider.translation().y,
                                h_width * 2.0,
                                h_height * 2.0,
                            );
                            let origin = rvec2(h_width, h_height);
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
        }

        // Draw rendered texture
        {
            let mut d = d.begin_shader_mode(if self.blur {
                &self.blur_shader
            } else {
                &self.def_shader
            });
            d.draw_texture_pro(
                self.ren_tex.texture(),
                rrect(
                    0,
                    0,
                    self.ren_tex.texture.width,
                    -self.ren_tex.texture.height,
                ),
                rrect(
                    0,
                    0,
                    self.ren_tex.texture.width / 2,
                    self.ren_tex.texture.height / 2,
                ),
                rvec2(0, 0), //rvec2(ren_tex.texture.width/2, ren_tex.texture.height/2),
                0.,
                Color::WHITE,
            );
        }

        // Draw UI
        {
            // Draw fps in top-left corner
            if self.draw_fps {
                d.draw_text_ex(
                    &self.font,
                    &format!("{:.1}", 1.0 / delta),
                    rvec2(1825., 0.),
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
                rvec2(0.0, 50.0 * line),
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
                rvec2(0.0, 50.0 * line),
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
                rvec2(0.0, 50.0 * line),
                50.0,
                0.0,
                Color::GREEN,
            );

            // Fuel
            if self.fuel_mode {
                if let Some(player) = &self.player_rc {
                    let mut player = player.borrow_mut();
                    player.level_completed = self.completed;
                    let fuel_text = format!("Fuel: {:.0}", player.fuel);
                    line += 1.0;
                    d.draw_text_ex(
                        &self.font,
                        &fuel_text,
                        rvec2(0.0, 50.0 * line),
                        50.0,
                        0.0,
                        Color::GREEN,
                    );
                }
            }

            // "Paused" text
            if self.paused {
                let text = "Paused";
                let mut text_position = self.camera.offset / 2.0; // center
                text_position += rvec2(-75.0, -230.0); // offset from center
                d.draw_text_ex(
                    &self.font,
                    text,
                    text_position,
                    50.0,
                    0.0,
                    Color::PINK,
                );
            }

            // Restart prompt
            if self.completed {
                let restart_text = if self
                    .player_rc
                    .as_deref()
                    .map(|p| p.borrow().failed)
                    .unwrap_or(true)
                {
                    "     Level failed\nPress R to restart"
                } else {
                    "Level completed!\nPress R to restart"
                };
                let mut text_position = self.camera.offset / 2.0; // center
                text_position += rvec2(-150.0, -130.0); // offset from center
                d.draw_text_ex(
                    &self.font,
                    restart_text,
                    text_position,
                    50.0,
                    0.0,
                    Color::GOLD,
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
    pub fn spawn_player(&mut self, position: NVector2, fuel: f32) {
        assert!(self.player_rc.is_none(), "Can't spawn a second player");
        let mut player = Player::new(self.player_tex.clone(), self.exhaust_tex.clone());
        player.fuel = fuel;
        player.fuel_mode = self.fuel_mode;

        let rigid_body = RigidBodyBuilder::new_dynamic()
            .translation(position)
            .can_sleep(false)
            .build();
        let collider = ColliderBuilder::capsule_y(25.0, 14.0)
            .position(Isometry::new(vector![0., -3.0], 0.0))
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

        let uuid = player.get_uuid();
        let player_rc = Rc::new(RefCell::new(player));
        self.process_objects.insert(uuid, player_rc.clone());
        self.draw_objects.insert(uuid, player_rc.clone());
        self.phys_objects.insert(uuid, player_rc.clone());
        self.player_rc = Some(player_rc);
    }

    /// Spawns an asteroid at given position
    pub fn spawn_asteroid(
        &mut self,
        position: NVector2,
        scale: f32,
        velocities: RigidBodyVelocity,
    ) {
        let mut asteroid = GameObject::new();
        asteroid.sprite = Some(Sprite::new(self.asteroid_tex.clone(), true, scale));

        let rigid_body = RigidBodyBuilder::new_dynamic()
            .translation(position)
            .rotation(position.x * position.y)
            .linvel(velocities.linvel)
            .angvel(velocities.angvel)
            .can_sleep(false)
            .build();
        let collider = ColliderBuilder::capsule_y(0.0, 40.0 * scale)
            .restitution(0.8)
            .density(2.0)
            .active_events(ActiveEvents::CONTACT_EVENTS)
            .build();

        let uuid = asteroid.get_uuid();

        asteroid.update_state(&rigid_body);

        let rigid_body_handle = self.rigid_body_set.insert(rigid_body);

        let col_handle = self.collider_set.insert_with_parent(
            collider,
            rigid_body_handle,
            &mut self.rigid_body_set,
        );
        self.asteroid_colliders.insert(col_handle, uuid);
        asteroid.set_body(rigid_body_handle);

        let asteroid_rc = Rc::new(RefCell::new(asteroid));
        self.process_objects.insert(uuid, asteroid_rc.clone());
        self.draw_objects.insert(uuid, asteroid_rc.clone());
        self.phys_objects.insert(uuid, asteroid_rc);
    }

    /// Spawns asteroids around given planet
    pub fn spawn_asteroids_around_planet(&mut self, planet_pos: NVector2, planet_radius: f32) {
        let asteroid_count = self.rng.gen_range(10..30);
        for _ in 0..asteroid_count {
            let rot = Rotation::new(self.rng.gen_range(0.0..2.0 * PI));
            let offset = rot * vector![1., 0.] * self.rng.gen_range(1.5..5.5) * planet_radius;
            let linvel = self.rng.gen_range(30.0..300.0)
                * vector![
                    self.rng.gen_range(-1.0..1.0f32),
                    self.rng.gen_range(-1.0..1.0f32)
                ]
                .normalize();
            let angvel = self.rng.gen_range(-10.0..10.0);
            let scale = self.rng.gen_range(0.2..0.6);
            self.spawn_asteroid(
                planet_pos + offset,
                scale,
                RigidBodyVelocity { linvel, angvel },
            );
        }
    }

    /// Spawns a planet at given position with given radius
    pub fn spawn_planet(
        &mut self,
        position: NVector2,
        radius: f32,
        color_a: Color,
        color_b: Color,
    ) {
        let mut planet = Planet::new(
            to_rv2(position),
            0.,
            radius,
            color_a,
            color_b,
            self.asteroid_tex.clone(),
        );

        let rigid_body = RigidBodyBuilder::new_static()
            .translation(position)
            .can_sleep(false)
            .build();
        let collider = ColliderBuilder::ball(radius).density(8.0).build();

        let rigid_body_handle = self.rigid_body_set.insert(rigid_body);
        self.collider_set
            .insert_with_parent(collider, rigid_body_handle, &mut self.rigid_body_set);
        planet.set_body(rigid_body_handle);

        let uuid = planet.get_uuid();
        let planet_rc = Rc::new(RefCell::new(planet));
        self.phys_objects.insert(uuid, planet_rc.clone());
        self.planet_objects.insert(uuid, planet_rc);
    }

    /// Spawns a gate at given position
    pub fn spawn_gate(&mut self, position: NVector2, rotation: f32) {
        let mut gate = Gate::new(
            self.gate_tex.clone(),
            self.gate_off_tex.clone(),
            self.gate_darker_tex.clone(),
        );
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

        let uuid = gate.get_uuid();
        let gate_rc = Rc::new(RefCell::new(gate));
        self.phys_objects.insert(uuid, gate_rc.clone());
        self.gate_objects.push(gate_rc);

        self.gate_count += 1;
    }

    /// Spawns a planet with gates around it
    pub fn spawn_planet_with_gates(&mut self, position: NVector2, radius: f32, gate_count: u16) {
        assert!(gate_count < 6, "Gate count must be less than 6");

        let hue = self.rng.gen::<f32>() * 250.;
        let sat = self.rng.gen::<f32>() * 0.3 + 0.5;
        let color_a = Color::color_from_hsv(hue, sat, 0.9);
        let hue = hue + 3.0;
        let sat = self.rng.gen::<f32>() * 0.3 + 0.5;
        let color_b = Color::color_from_hsv(hue, sat, 0.8);

        self.spawn_planet(position, radius, color_a, color_b);

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
        let mut planets: Vec<(NVector2, f32)> = Vec::new();

        let radius_range = 300.0..700.0;

        let mut gates_left = num_gates;

        let mut last_position: NVector2 = vector![0., 0.];
        let mut last_radius = 0.;
        while gates_left > 0 {
            let mut position_valid = false;
            let radius = self.rng.gen_range(radius_range.clone());
            let mut distance = (last_radius + radius) * (3.0 + self.rng.gen::<f32>());
            let mut pos: NVector2 = vector![0., 0.];
            while !position_valid {
                let angle = self.rng.gen::<f32>() * PI * 2.0;
                let rot = Rotation::new(angle);
                let offset = rot.into_inner() * distance;
                distance *= 1.05;
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
                ((self.rng.gen_range(1..6) + self.rng.gen_range(0..6)) as f32 * 0.5).ceil() as u16;
            if self.rng.gen_bool(0.3) {
                gate_count = 0;
            } else {
                last_radius = radius;
                last_position = pos;
            }
            gate_count = gate_count.min(gates_left);
            gates_left -= gate_count;
            self.spawn_planet_with_gates(pos, radius, gate_count);
            self.spawn_asteroids_around_planet(pos, radius);
            planets.push((pos, radius));
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum GameAction {
    Menu,
    Restart,
    NewSeed,
    Quit,
}
