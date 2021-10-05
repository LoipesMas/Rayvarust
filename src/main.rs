use ::core::cell::RefCell;
use raylib::prelude::*;
use std::rc::Rc;

mod math;

mod game_object;
use game_object::*;

mod physics;
use physics::*;

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
        zoom: 1.,
    };

    let player = Player::new(Rc::clone(&player_tex_ref));

    let player_rc = Rc::new(RefCell::new(player));
    process_objects.push(player_rc.clone());
    draw_objects.push(player_rc.clone());
    phys_objects.push(player_rc.clone());

    let center = Vector2 {
        x: 40. * 4.5,
        y: 50. * 4.5,
    };

    // Spawn 100 astronauts
    for i in 0..10 {
        for j in 0..10 {
            let mut pl = GameObject::new();
            pl.sprite = Some(Sprite::new(Rc::clone(&astronaut_tex_ref), true, 0.3));
            pl.physics_body = Some(PhysicsBody::new(CollisionShape::Circle(
                Vector2::zero(),
                10.0,
            )));
            let pos = Vector2::new(40. * i as f32, 50. * j as f32);
            pl.set_position(pos);

            let pl_rc = Rc::new(RefCell::new(pl));
            let mut vel = center - pos;
            if vel.length_sqr() > 0. {
                vel.normalize();
            }
            vel *= 60.;
            pl_rc.borrow_mut().get_body_mut().add_linear_velocity(vel);
            draw_objects.push(pl_rc.clone());
            process_objects.push(pl_rc.clone());
            phys_objects.push(pl_rc);
        }
    }

    while !rl.window_should_close() {
        let delta = rl.get_frame_time();

        // Processing
        for object in &process_objects {
            object.borrow_mut().process(&mut rl, delta);
        }

        // Physics
        for i in 0..phys_objects.len() {
            let mut object = phys_objects[i].borrow_mut();
            object.physics_process(delta);
            let body = object.get_body();
            for other_object in phys_objects[i + 1..].iter() {
                let other_object = other_object.borrow();
                let other_body = other_object.get_body();
                if body.check_body_collision(other_body) {
                    //TODO: Do something with collision
                }
            }
        }

        camera.target = player_rc.borrow().get_position();

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
                    let object = object.borrow();
                    let body = object.get_body();
                    body.debug_draw(&mut mode);
                }
            }
        }

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
