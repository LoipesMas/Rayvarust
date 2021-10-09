use raylib::prelude::*;
mod math;

mod game_object;

mod game;
use game::Game;

use rapier2d::prelude::*;

fn main() {
    let mut the_game = Game::new();

    the_game.spawn_planet(vector![500., -1400.], 400.);
    the_game.spawn_planet(vector![800., 2300.], 600.);

    the_game.spawn_gate(vector![100., 0.0]);

    the_game.spawn_asteroids();

    the_game.spawn_player(vector![-100., 0.]);

    the_game.spawn_gate(vector![-300., 0.0]);

    the_game.spawn_gate(vector![-500., 0.0]);

    the_game.run();
}
