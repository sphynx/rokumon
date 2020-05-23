mod ai;
mod board;
mod card;
mod console_ui;
mod coord;
mod game;
mod parsers;
mod perft;
mod play;

use ai::AlphaBetaAI;
use board::Layout;
use card::Deck;
use console_ui::Human;
use failure::{bail, Fallible};
use game::{Game, Rules};
use perft::*;
use play::RandomAI;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use std::time::Instant;
use structopt::StructOpt;

#[derive(Debug)]
enum Mode {
    Perft,
    ParallelPerft,
    Play,
    Match,
}

impl FromStr for Mode {
    type Err = failure::Error;
    fn from_str(s: &str) -> Fallible<Self> {
        use Mode::*;
        match s.to_lowercase().as_str() {
            "perft" => Ok(Perft),
            "par_perft" => Ok(ParallelPerft),
            "play" => Ok(Play),
            "match" => Ok(Match),
            _ => bail!("Can't parse play mode: {}", s),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
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
    /// Run mode (use `play` to play the game). perft is used to test performance of the move generator.
    #[structopt(short, long, default_value = "play", help = "play | perft | par_perft")]
    mode: Mode,

    /// Opponents for the game.
    #[structopt(
        short,
        long,
        default_value = "HumanAI",
        help = "HumanHuman | HumanAI | AIHuman | AIAI | RandomRandom"
    )]
    opponents: Opponents,

    /// Depth for performance tests.
    #[structopt(long, default_value = "5")]
    perft_depth: usize,

    /// Cards to be used in the game (g - gold, j - jade)
    #[structopt(long, default_value = "gggjjjj")]
    cards: String,

    /// If we don't want to shuffle the deck before starting the game.
    #[structopt(long)]
    no_shuffle: bool,

    /// Layout: Bricks7 or Square6 (for Act I).
    #[structopt(short, long, default_value = "bricks7")]
    layout: Layout,

    /// How deep AI should analyse the position (in plies, i.e. half-moves).
    #[structopt(long)]
    ai_depth: Option<u32>,

    /// How much time AI should use for its moves (in seconds).
    #[structopt(long)]
    ai_duration: Option<u64>,

    /// Whether AI should run its analysis to full completion before making the move.
    #[structopt(long)]
    ai_to_completion: bool,

    /// How deep second AI should analyse the position (in plies, i.e. half-moves).
    #[structopt(long)]
    second_ai_depth: Option<u32>,

    /// How much time second AI should use for its moves (in seconds).
    #[structopt(long)]
    second_ai_duration: Option<u64>,

    /// Whether second AI should run its analysis to full completion before making the move.
    #[structopt(long)]
    second_ai_to_completion: bool,

    /// Number of matches to play (for AI vs AI games).
    #[structopt(long, default_value = "10")]
    samples: u32,

    /// Allows 'Fight' move in the game rules (disabled by default).
    #[structopt(short = "f", long)]
    enable_fight_move: bool,

    /// Allows 'Surprise' move in the game rules (disabled by default).
    #[structopt(short = "s", long)]
    enable_surprise_move: bool,
}

impl Display for Opt {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Options: mode={:?}, opponents={:?}, cards={:?}, no_shuffle={}, samples={}, layout={:?}, with_fight={}, \
             with_surprise={}, ai_duration={:?}, ai_depth={:?}, second_ai_duration={:?}, second_ai_depth={:?}",
            self.mode,
            self.opponents,
            self.cards,
            self.no_shuffle,
            self.samples,
            self.layout,
            self.enable_fight_move,
            self.enable_surprise_move,
            self.ai_duration,
            self.ai_depth,
            self.second_ai_duration,
            self.second_ai_depth,
        )
    }
}

fn play_game(opt: &Opt, rules: &Rules) -> i8 {
    let cards_spec = opt.cards.as_str();
    let deck = if opt.no_shuffle {
        Deck::ordered(cards_spec)
    } else {
        Deck::shuffled(cards_spec)
    }
    .unwrap();

    let game = Game::new(Layout::Bricks7, deck, rules.clone());
    match opt.opponents {
        Opponents::HumanHuman => play::play_game(game, Human, Human),
        Opponents::RandomRandom => play::play_game(game, RandomAI, RandomAI),
        Opponents::HumanAI => play::play_game(game, Human, mk_bot(false, &opt)),
        Opponents::AIHuman => play::play_game(game, mk_bot(true, &opt), Human),
        Opponents::AIAI => play::play_game(game, mk_bot(true, &opt), mk_bot(false, &opt)),
    }
}

fn mk_bot(for_first_player: bool, opt: &Opt) -> AlphaBetaAI {
    let (duration_option, depth_option, to_completion_option) = if for_first_player || opt.opponents != Opponents::AIAI
    {
        (opt.ai_duration, opt.ai_depth, opt.ai_to_completion)
    } else {
        (opt.second_ai_duration, opt.second_ai_depth, opt.second_ai_to_completion)
    };

    if to_completion_option {
        AlphaBetaAI::to_completion(for_first_player)
    } else if let Some(dur) = duration_option {
        AlphaBetaAI::with_duration(for_first_player, dur)
    } else if let Some(depth) = depth_option {
        AlphaBetaAI::with_depth(for_first_player, depth)
    } else {
        AlphaBetaAI::with_duration(for_first_player, 2)
    }
}

fn play_match(opt: &Opt, rules: &Rules) {
    let n = opt.samples;
    println!("Starting a match of {} games", n);
    let mut wins = 0;
    let mut draws = 0;
    let mut losses = 0;
    for ix in 0..n {
        println!();
        println!("Starting game {}", ix + 1);
        let res = play_game(opt, rules);
        match res {
            1 => wins += 1,
            0 => draws += 1,
            -1 => losses += 1,
            _ => panic!("Unexpected game result: {}", res),
        }
        println!(
            "Match status so far: played {} games: {} : {} : {}",
            ix + 1,
            wins,
            draws,
            losses
        );
        println!();
    }
    println!("Played {} games in total: {} : {} : {}", n, wins, draws, losses);
}

fn main() -> Fallible<()> {
    let opt = Opt::from_args();
    println!("{}", opt);

    let rules = Rules::new(opt.enable_fight_move, opt.enable_surprise_move);
    match &opt.mode {
        Mode::Play => {
            play_game(&opt, &rules);
        }
        Mode::Match => {
            play_match(&opt, &rules);
        }
        Mode::Perft | Mode::ParallelPerft => {
            let max_depth = opt.perft_depth;
            let cards_spec = opt.cards.as_str();
            let deck = if opt.no_shuffle {
                Deck::ordered(cards_spec)
            } else {
                Deck::shuffled(cards_spec)
            }?;

            let mut game = Game::new(Layout::Bricks7, deck, rules);
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
