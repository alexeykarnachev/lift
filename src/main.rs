#![allow(dead_code)]
#![allow(unused_variables)]

use game::*;
use vec::Vec2;

mod frame;
mod game;
mod input;
mod renderer;
mod utils;
mod vec;

fn main() {
    let window_size: Vec2<u32> = Vec2::new(1500, 800);

    let game = Box::leak(Box::new(Game::new(
        window_size,
        "./assets/sprites/atlas.json",
        "./assets/sprites/atlas.png",
    )));
    game.start();
}
