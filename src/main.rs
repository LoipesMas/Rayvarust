use ::core::cell::RefCell;
use raylib::prelude::*;
use std::rc::Rc;

use rapier2d::prelude::*;

mod math;
use math::{lerp, to_nv2, to_rv2};

mod game_object;
use game_object::*;

// Color of debug collider
const COLL_COLOR: Color = Color {
    r: 70,
    g: 200,
    b: 70,
    a: 200,
};

fn main() {
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

    let bg = color::rcolor(47, 40, 70, 255);

    // Initialize physics
    let mut rigid_body_set = RigidBodySet::new();
    let mut collider_set = ColliderSet::new();

    /* Create other structures necessary for the simulation. */
    let gravity = vector![0.0, 0.0];
    let integration_parameters = IntegrationParameters::default();
    let mut physics_pipeline = PhysicsPipeline::new();
    let mut island_manager = IslandManager::new();
    let mut broad_phase = BroadPhase::new();
    let mut narrow_phase = NarrowPhase::new();
    let mut joint_set = JointSet::new();
    let mut ccd_solver = CCDSolver::new();
    let physics_hooks = ();
    let event_handler = ();

    let mut process_objects: Vec<Rc<RefCell<dyn Processing>>> = Vec::new();
    let mut draw_objects: Vec<Rc<RefCell<dyn Drawable>>> = Vec::new();
    let mut phys_objects: Vec<Rc<RefCell<dyn PhysicsObject>>> = Vec::new();

    let font = rl
        .load_font(&thread, "resources/fonts/RobotoMono-Regular.ttf")
        .expect("Couldn't load font");

    let player_tex = rl
        .load_texture(&thread, "resources/textures/spaceship.png")
        .expect("Couldn't load spaceship.png");
    let player_tex_ref = Rc::new(RefCell::new(player_tex));

    let astronaut_tex = rl
        .load_texture(&thread, "resources/textures/astronaut.png")
        .expect("Couldn't load astronaut.png");
    let astronaut_tex_ref = Rc::new(RefCell::new(astronaut_tex));

    let mut camera = Camera2D {
        offset: Vector2::new(window_width as f32 / 2.0, window_height as f32 / 2.0),
        target: Vector2::new(0., 0.),
        rotation: 0.,
        zoom: 0.66,
    };

    let mut player = Player::new(Rc::clone(&player_tex_ref));

    let rigid_body = RigidBodyBuilder::new_dynamic()
        .translation(vector![0.0, 10.0])
        .build();
    let collider = ColliderBuilder::capsule_y(20.0, 20.0)
        .position(Isometry::new(vector![0., 0.0], 0.0))
        .density(2.0)
        .build();
    let player_body_handle = rigid_body_set.insert(rigid_body);
    collider_set.insert_with_parent(collider, player_body_handle, &mut rigid_body_set);
    player.set_body(player_body_handle);

    let player_rc = Rc::new(RefCell::new(player));
    process_objects.push(player_rc.clone());
    draw_objects.push(player_rc.clone());
    phys_objects.push(player_rc.clone());

    let center = vector![63. * 4.5, 50. * 4.5];

    // Spawn 100 astronauts
    for i in 0..10 {
        for j in 0..10 {
            let mut pl = GameObject::new();
            pl.sprite = Some(Sprite::new(Rc::clone(&astronaut_tex_ref), true, 0.3));
            let pos = vector![60. * i as f32, 50. * j as f32];

            let mut rigid_body = RigidBodyBuilder::new_dynamic().translation(pos).build();
            let collider = ColliderBuilder::capsule_y(4.0, 8.0).build();

            let mut vel = center - pos;
            vel.normalize_mut();
            vel *= 30.;
            rigid_body.set_linvel(vel, true);

            let rigid_body_handle = rigid_body_set.insert(rigid_body);
            collider_set.insert_with_parent(collider, rigid_body_handle, &mut rigid_body_set);
            pl.set_body(rigid_body_handle);

            let pl_rc = Rc::new(RefCell::new(pl));
            process_objects.push(pl_rc.clone());
            draw_objects.push(pl_rc.clone());
            phys_objects.push(pl_rc.clone());
        }
    }

    while !rl.window_should_close() {
        let delta = rl.get_frame_time();

        // Processing
        for object in &process_objects {
            object.borrow_mut().process(&mut rl, delta);
        }

        for object in phys_objects.iter_mut() {
            let body = &mut rigid_body_set[*object.borrow().get_body()];
            object.borrow_mut().physics_process(delta, body);
        }

        // Physics
        physics_pipeline.step(
            &gravity,
            &integration_parameters,
            &mut island_manager,
            &mut broad_phase,
            &mut narrow_phase,
            &mut rigid_body_set,
            &mut collider_set,
            &mut joint_set,
            &mut ccd_solver,
            &physics_hooks,
            &event_handler,
        );

        camera.rotation = -player_rc.borrow().get_rotation() * RAD2DEG as f32;

        for object in phys_objects.iter_mut() {
            let body = &rigid_body_set[*object.borrow().get_body()];
            object.borrow_mut().update_state(body);
        }

        camera.target = to_rv2(lerp(to_nv2(camera.target), to_nv2(player_rc.borrow().get_position()), 0.2));

        let mut d = rl.begin_drawing(&thread);

        {
            let mut mode = d.begin_mode2D(camera);
            mode.clear_background(bg);

            // Rendering objects
            for object in &draw_objects {
                object.borrow_mut().draw(&mut mode);
            }

            // Draw collision
            if draw_collisions {
                for object in phys_objects.iter() {
                    let body = &rigid_body_set[*object.borrow().get_body()];
                    for collider in body.colliders() {
                        let collider = &collider_set[*collider];
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
        if draw_fps {
            d.draw_text_ex(
                &font,
                &(1. / delta).to_string(),
                Vector2::zero(),
                50.0,
                1.0,
                Color::WHITE,
            );
        }
    }
}
