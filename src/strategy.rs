use crate::coord::Coord;
use crate::game::{Game, GameMove};
use crate::parsers;

use rubot::{Bot, Depth, Logger, ToCompletion};
use std::io;
use std::time::Duration;

pub trait Strategy {
    fn get_move(&mut self, game: &Game) -> GameMove<Coord>;
}

pub struct Human;

impl Strategy for Human {
    fn get_move(&mut self, game: &Game) -> GameMove<Coord> {
        println!("Your turn, please enter your move:");
        loop {
            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("Failed to read input");
            match parsers::parse_move(input.trim_end()) {
                Ok(mov) => match game.convert_move_coords(&mov) {
                    Ok(m) => break m,
                    Err(_) => println!("Invalid user coordinate. Please try again:"),
                },
                Err(_) => println!("Can't parse move. Please try again:"),
            }
        }
    }
}

pub struct RandomAI;

impl Strategy for RandomAI {
    fn get_move(&mut self, game: &Game) -> GameMove<Coord> {
        game.random_move()
    }
}

pub struct AlphaBetaAI {
    duration: u64,
    depth: u32,
    bot: Bot<Game>,
}

impl AlphaBetaAI {
    pub fn with_duration(for_first_player: bool, duration: u64) -> Self {
        Self {
            bot: Bot::new(for_first_player),
            duration,
            depth: 0,
        }
    }

    pub fn with_depth(for_first_player: bool, depth: u32) -> Self {
        Self {
            bot: Bot::new(for_first_player),
            duration: 0,
            depth,
        }
    }

    pub fn to_completion(for_first_player: bool) -> Self {
        Self {
            bot: Bot::new(for_first_player),
            duration: 0,
            depth: 0,
        }
    }
}

impl Strategy for AlphaBetaAI {
    fn get_move(&mut self, game: &Game) -> GameMove<Coord> {
        macro_rules! log_ai {
            ($condition:expr) => {
                let mut logger = Logger::new($condition);
                let res = self.bot.select(&game, &mut logger).unwrap();
                println!(
                    "AI log: steps: {}, depth: {}, completed: {}, duration: {:?}",
                    logger.steps(),
                    logger.depth(),
                    logger.completed(),
                    logger.duration()
                );
                return res;
            };
        }

        if self.duration != 0 {
            log_ai!(Duration::from_secs(self.duration));
        } else if self.depth != 0 {
            log_ai!(Depth(self.depth));
        } else {
            log_ai!(ToCompletion);
        }
    }
}
