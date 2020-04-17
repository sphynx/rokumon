mod game;
mod parsers;

use failure::Fallible;
use game::{Game, Layout, Deck};

fn main() -> Fallible<()> {
    println!("Rokumon perft, v0.1");
    let deck = Deck::ordered("gggjjjj")?;
    let mut game = Game::new(Layout::Bricks7, deck);
    let depth = 4;
    for d in 1..=depth {
        let perft = game.perft(d)?;
        println!("perft({}) = {}", d, perft);
    }

    Ok(())
}
