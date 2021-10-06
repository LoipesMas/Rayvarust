use raylib::prelude::*;
mod math;

mod game_object;

mod game;
use game::Game;



fn main() {
    let mut the_game = Game::new();

    the_game.spawn_player();

    the_game.spawn_asteroids();

    the_game.run();
}
