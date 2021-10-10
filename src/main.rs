use raylib::prelude::*;
mod math;

mod game_object;

mod game;
use game::Game;

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

    let mut the_game = Game::new(&mut rl, &thread, window_width, window_height);

    the_game.spawn_many_planets_with_gates(6);

    the_game.spawn_player(vector![0., 0.]);

    the_game.run();

    the_game.unload();

    drop(the_game);
}
