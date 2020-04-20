mod ai;
mod board;
mod card;
mod coord;
mod game;
mod parsers;
mod perft;

use ai::Opponent;
use board::Layout;
use card::Deck;
use failure::{bail, Fallible};
use game::{Game, Rules};
use perft::*;
use rubot::{Depth, ToCompletion};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use std::time::{Duration, Instant};
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
    Analyse
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
            "analyse" | "analyze" => Ok(Analyse),
            _ => bail!("Can't parse opponents specification: {}", s),
        }
    }
}

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, long, default_value = "play", help = "perft | par_perft | play")]
    mode: Mode,

    #[structopt(short, long, default_value = "AIAI", help = "HumanHuman | HumanAI | AIHuman | AIAI")]
    opponents: Opponents,

    #[structopt(short, long, default_value = "5")]
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

    let deck = Deck::ordered(opt.cards.as_str())?;
    let rules = Rules::new(opt.enable_fight_move, opt.enable_surprise_move);
    let max_depth = opt.depth;

    let mut game = Game::new(Layout::Bricks7, deck, rules);

    match &opt.mode {
        Mode::Play => match opt.opponents {
            Opponents::Analyse => {
                game.apply_move_str("place r2 at r2c2")?;
                game.apply_move_str("place b1 at r2c3")?;
                game.apply_move_str("place r2 at r1c2")?;
                println!("Analysing this position: {}", &game);
                let ai = Opponent::new_ai(false, &ToCompletion);
                ai::play_game(game, Opponent::Human, ai);
            }
            Opponents::HumanHuman => ai::play_game::<Depth>(game, Opponent::Human, Opponent::Human),
            Opponents::HumanAI => {
                let condition = Duration::from_secs(3);
                let ai2 = Opponent::new_ai(false, &condition);
                ai::play_game(game, Opponent::Human, ai2);
            }
            Opponents::AIHuman => {
                let condition = Duration::from_secs(3);
                let ai1 = Opponent::new_ai(true, &condition);
                ai::play_game(game, ai1, Opponent::Human);
            }

            Opponents::AIAI => {
                let condition = Duration::from_secs(60);
                let ai1 = Opponent::new_ai(true, &condition);
                let ai2 = Opponent::new_ai(false, &condition);
                ai::play_game(game, ai1, ai2);
            }
        },

        Mode::Perft | Mode::ParallelPerft => {
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
