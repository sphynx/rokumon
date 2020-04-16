mod game;
mod parsers;

use failure::Fallible;
use game::Game;

fn main() -> Fallible<()> {
    println!("Rokumon perft, v0.1");

    let mut game = Game::new();
    let depth = 5;
    for d in 1..=depth {
        let perft = game.perft(d)?;
        println!("perft({}) = {}", d, perft);
    }

    Ok(())
}
