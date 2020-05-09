mod ai;
mod board;
mod card;
mod coord;
mod game;
mod parsers;
mod perft;
mod play;
mod strategy;

use board::Layout;
use card::Deck;
use failure::{bail, Fallible};
use game::{Game, Rules};
use perft::*;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use std::time::Instant;
use strategy::{AlphaBetaAI, Human, RandomAI};
use structopt::StructOpt;

#[derive(Debug)]
enum Mode {
    Perft,
    ParallelPerft,
    Play,
}

impl FromStr for Mode {
    type Err = failure::Error;
    fn from_str(s: &str) -> Fallible<Self> {
        use Mode::*;
        match s.to_lowercase().as_str() {
            "perft" => Ok(Perft),
            "par_perft" => Ok(ParallelPerft),
            "play" => Ok(Play),
            _ => bail!("Can't parse play mode: {}", s),
        }
    }
}

#[derive(Debug)]
enum Opponents {
    HumanHuman,
    HumanAI,
    AIHuman,
    AIAI,
    RandomRandom,
}

impl FromStr for Opponents {
    type Err = failure::Error;
    fn from_str(s: &str) -> Fallible<Self> {
        use Opponents::*;
        match s.to_lowercase().as_str() {
            "humanhuman" => Ok(HumanHuman),
            "humanai" => Ok(HumanAI),
            "aihuman" => Ok(AIHuman),
            "aiai" => Ok(AIAI),
            "rr" | "randomrandom" => Ok(RandomRandom),
            _ => bail!("Can't parse opponents specification: {}", s),
        }
    }
}

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, long, default_value = "play", help = "perft | par_perft | play")]
    mode: Mode,

    #[structopt(
        short,
        long,
        default_value = "RR",
        help = "HumanHuman | HumanAI | AIHuman | AIAI | RR"
    )]
    opponents: Opponents,

    #[structopt(short, long, default_value = "5")]
    depth: usize,

    #[structopt(long, default_value = "gggjjjj")]
    cards: String,

    #[structopt(long)]
    shuffle: bool,

    #[structopt(short, long, default_value = "bricks7")]
    layout: Layout,

    #[structopt(long)]
    ai_depth: Option<u32>,

    #[structopt(long)]
    ai_duration: Option<u64>,

    #[structopt(long)]
    second_ai_depth: Option<u32>,

    #[structopt(long)]
    second_ai_duration: Option<u64>,

    #[structopt(short = "f", long)]
    enable_fight_move: bool,

    #[structopt(short = "s", long)]
    enable_surprise_move: bool,
}

impl Display for Opt {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Options: depth={}, mode={:?}, opponents={:?}, cards={:?}, layout={:?}, with_fight={}, with_surprise={}",
            self.depth,
            self.mode,
            self.opponents,
            self.cards,
            self.layout,
            self.enable_fight_move,
            self.enable_surprise_move
        )
    }
}

fn main() -> Fallible<()> {
    let opt = Opt::from_args();
    println!("{}", opt);

    let cards_spec = opt.cards.as_str();
    let deck = if opt.shuffle {
        Deck::shuffled(cards_spec)
    } else {
        Deck::ordered(cards_spec)
    }?;

    let rules = Rules::new(opt.enable_fight_move, opt.enable_surprise_move);

    let mut game = Game::new(Layout::Bricks7, deck, rules);

    match &opt.mode {
        Mode::Play => match opt.opponents {
            Opponents::HumanHuman => play::play_game(game, Human, Human),
            Opponents::RandomRandom => play::play_game(game, RandomAI, RandomAI),

            Opponents::HumanAI => {
                let for_first_player = false;
                let ai = if let Some(dur) = opt.ai_duration {
                    AlphaBetaAI::with_duration(for_first_player, dur)
                } else if let Some(d) = opt.ai_depth {
                    AlphaBetaAI::with_depth(for_first_player, d)
                } else {
                    AlphaBetaAI::to_completion(for_first_player)
                };
                play::play_game(game, Human, ai)
            }

            Opponents::AIHuman => {
                let for_first_player = true;
                let ai = if let Some(dur) = opt.ai_duration {
                    AlphaBetaAI::with_duration(for_first_player, dur)
                } else if let Some(d) = opt.ai_depth {
                    AlphaBetaAI::with_depth(for_first_player, d)
                } else {
                    AlphaBetaAI::to_completion(for_first_player)
                };
                play::play_game(game, ai, Human)
            }

            Opponents::AIAI => {
                let for_first_player = true;
                let first_ai = if let Some(dur) = opt.ai_duration {
                    AlphaBetaAI::with_duration(for_first_player, dur)
                } else if let Some(d) = opt.ai_depth {
                    AlphaBetaAI::with_depth(for_first_player, d)
                } else {
                    AlphaBetaAI::to_completion(for_first_player)
                };

                let for_first_player = false;
                let second_ai = if let Some(dur) = opt.second_ai_duration {
                    AlphaBetaAI::with_duration(for_first_player, dur)
                } else if let Some(d) = opt.second_ai_depth {
                    AlphaBetaAI::with_depth(for_first_player, d)
                } else {
                    AlphaBetaAI::to_completion(for_first_player)
                };

                play::play_game(game, first_ai, second_ai)
            }
        },

        Mode::Perft | Mode::ParallelPerft => {
            let max_depth = opt.depth;
            for depth in 1..=max_depth {
                let now = Instant::now();
                let perft = match &opt.mode {
                    Mode::Perft => perft(&mut game, depth),
                    Mode::ParallelPerft => parallel_perft(&game, depth),
                    _ => unreachable!(),
                };
                let elapsed = now.elapsed();
                let ratio = perft as f64 / now.elapsed().as_micros() as f64 * 1e6;
                println!(
                    "perft({}): {:9}, time: {:>8} speed: {:.0} moves/s",
                    depth,
                    perft,
                    format!("{:.0?},", elapsed),
                    ratio
                );
            }
        }
    }

    Ok(())
}
