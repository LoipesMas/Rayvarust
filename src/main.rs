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

fn main() {
    let levels_lengths: Vec<u16> = vec![8, 16, 24];
    let levels_seeds: Vec<u64> = vec![4538, 1337, 22664];
    let levels_fuels: Vec<f32> = vec![333., 932., 1416.];

    let window_width: i16 = 960 * 2;
    let window_height: i16 = 540 * 2;
    let (mut rl, thread) = raylib::init()
        .size(window_width.into(), window_height.into())
        .title("Rayvarust")
        .fullscreen()
        .vsync()
        .build();

    rl.set_exit_key(None);

    let mut restart = false;
    let mut quit = false;
    let mut action = MenuAction::Start(0, true, false);
    let mut random_levels = false;
    let mut fuel_mode = false;

    while !quit {
        if !restart {
            let mut menu = Menu::new(
                &mut rl,
                &thread,
                window_width,
                window_height,
                random_levels,
                fuel_mode,
            );
            action = menu.run();
        }
        restart = false;

        match action {
            MenuAction::Start(level, random, fuel) => {
                random_levels = random;
                fuel_mode = fuel;
                let length = levels_lengths[level];
                let window_width = rl.get_screen_width() as i16;
                let window_height = rl.get_screen_height() as i16;
                let seed = if random {
                    thread_rng().gen::<u16>() as u64
                } else {
                    levels_seeds[level]
                };
                let mut the_game = Game::new(
                    &mut rl,
                    &thread,
                    window_width,
                    window_height,
                    seed,
                    fuel_mode,
                );

                the_game.spawn_many_planets_with_gates(length);

                the_game.spawn_player(vector![0., 0.], levels_fuels[level]);

                let action = the_game.run();
                match action {
                    GameAction::Menu => {}
                    GameAction::Restart => {
                        restart = true;
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
