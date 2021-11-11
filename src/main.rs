#![windows_subsystem = "windows"]

use raylib::prelude::*;
mod math;

mod game_object;

mod game;
use game::{Game, GameAction};

mod menu;
use menu::{Menu, MenuAction};

use rapier2d::prelude::*;

use rand::prelude::*;

const SHIP_NAMES: [&str; 5] = ["sr", "sb", "sg", "sp", "sy"];

const FUEL_MULTIPLIER: f32 = 42.0;

fn main() {
    let window_width: i16 = 1920;
    let window_height: i16 = 1080;
    let (mut rl, thread) = raylib::init()
        .size(window_width.into(), window_height.into())
        .title("Rayvarust")
        .fullscreen()
        .vsync()
        .build();

    rl.set_target_fps(60);
    rl.set_exit_key(None);

    let mut restart = false;
    let mut quit = false;
    let mut selected_length = 6;
    let mut action = MenuAction::Start(selected_length, true, false);
    let mut random_levels = false;
    let mut fuel_mode = false;
    let mut seed = 0;
    let mut selected_ship = 0;

    while !quit {
        if !restart {
            let mut menu = Menu::new(
                &mut rl,
                &thread,
                window_width,
                window_height,
                random_levels,
                fuel_mode,
                selected_ship,
                selected_length.into(),
            );
            action = menu.run();
            menu.unload();
            selected_ship = menu.selected_ship;
            seed = 0;
        }
        restart = false;

        match action {
            MenuAction::Start(length, random, fuel) => {
                selected_length = length;
                random_levels = random;
                fuel_mode = fuel;
                let window_width = rl.get_screen_width() as i16;
                let window_height = rl.get_screen_height() as i16;
                if seed == 0 {
                    seed = if random {
                        thread_rng().gen::<u16>() as u64
                    } else {
                        length as u64
                    };
                }
                let mut the_game = Game::new(
                    &mut rl,
                    &thread,
                    window_width,
                    window_height,
                    seed,
                    fuel_mode,
                    selected_ship,
                );

                the_game.spawn_many_planets_with_gates(length);

                the_game.spawn_player(vector![0., 0.], FUEL_MULTIPLIER * length as f32);

                let action = the_game.run();
                match action {
                    GameAction::Menu => {}
                    GameAction::Restart => {
                        restart = true;
                    }
                    GameAction::NewSeed => {
                        restart = true;
                        seed = 0;
                    }
                    GameAction::Quit => quit = true,
                }

                the_game.unload();

                drop(the_game);
            }
            MenuAction::Quit => {
                break;
            }
        }

        // Hack to update key presses
        #[allow(unused_must_use)]
        {
            rl.begin_drawing(&thread);
        }

        if rl.window_should_close() {
            break;
        }
    }
}
