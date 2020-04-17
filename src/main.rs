mod game;
mod parsers;

use failure::Fallible;
use game::{Game, Layout, Deck};
use std::time::Instant;

fn main() -> Fallible<()> {
    println!("Rokumon perft, v0.1");
    let deck = Deck::ordered("gggjjjj")?;
    let mut game = Game::new(Layout::Bricks7, deck);
    let depth = 7;

    for d in 1..=depth {
        let now = Instant::now();
        let perft = game.perft(d)?;
        let elapsed_seconds = now.elapsed().as_secs();
        let ratio = perft as f64 / now.elapsed().as_micros() as f64 * 1000000.0;
        println!("perft({}) = {}, elapsed: {}s, {:.0} moves/s", d, perft, elapsed_seconds, ratio);
    }

    Ok(())
}
