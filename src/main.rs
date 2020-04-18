mod game;
mod parsers;

use failure::{bail, Fallible};
use game::{Deck, Game, Layout, Rules};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use std::time::Instant;
use structopt::StructOpt;

#[derive(Debug)]
enum PlayMode {
    Perft,
}

impl FromStr for PlayMode {
    type Err = failure::Error;
    fn from_str(s: &str) -> Fallible<Self> {
        if s.to_lowercase() == "perft" {
            Ok(PlayMode::Perft)
        } else {
            bail!("can't parse play mode: {}", s);
        }
    }
}

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, long, default_value = "perft")]
    mode: PlayMode,

    #[structopt(short, long)]
    depth: usize,

    #[structopt(long, default_value = "gggjjjj")]
    cards: String,

    #[structopt(short, long, default_value = "bricks7")]
    layout: Layout,

    #[structopt(short = "f", long)]
    enable_fight_move: bool,

    #[structopt(short = "s", long)]
    enable_surprise_move: bool,
}

impl Display for Opt {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Options: depth={}, mode={:?}, cards={:?}, layout={:?}, with_fight={}, with_surprise={}",
            self.depth, self.mode, self.cards, self.layout, self.enable_fight_move, self.enable_surprise_move
        )
    }
}

fn main() -> Fallible<()> {
    println!("Rokumon, v0.1");

    let opt = Opt::from_args();
    println!("{}", opt);

    let deck = Deck::ordered(opt.cards.as_str())?;
    let rules = Rules::new(opt.enable_fight_move, opt.enable_surprise_move);
    let depth = opt.depth;

    let mut game = Game::new(Layout::Bricks7, deck, rules);

    for d in 1..=depth {
        let now = Instant::now();
        let perft = game.perft(d)?;
        let elapsed_seconds = now.elapsed().as_secs();
        let ratio = perft as f64 / now.elapsed().as_micros() as f64 * 1e6;
        println!(
            "perft({}) = {}, elapsed: {}s, {:.0} moves/s",
            d, perft, elapsed_seconds, ratio
        );
    }

    Ok(())
}
