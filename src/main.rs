use raylib::prelude::*;
mod math;

mod game_object;

mod game;
use game::{Game, GameAction};

mod menu;
use menu::{Menu, MenuAction};

use rapier2d::prelude::*;

fn main() {
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

    while !quit {
        let ret = MenuAction::Start;
        if !restart {
            let mut menu = Menu::new(&mut rl, &thread, window_width, window_height);
            let ret = menu.run();

            if ret == MenuAction::Quit {
                break;
            }
        }
        restart = false;

        if ret == MenuAction::Start {
            let window_width = rl.get_screen_width() as i16;
            let window_height = rl.get_screen_height() as i16;
            let mut the_game = Game::new(&mut rl, &thread, window_width, window_height);

            the_game.spawn_many_planets_with_gates(24);

            the_game.spawn_player(vector![0., 0.]);

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
