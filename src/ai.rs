use rubot::{self, Bot, IntoRunCondition, Logger};

use std::i32;
// use std::time::Duration;

use crate::coord::Coord;
use crate::game::{Game, GameMove, GameResult};
use crate::parsers;

use std::io;

impl rubot::Game for Game {
    type Player = bool;
    type Fitness = i32;
    type Action = GameMove<Coord>;
    type Actions = Vec<GameMove<Coord>>;

    fn actions(&self, player: Self::Player) -> (bool, Self::Actions) {
        (player == self.player1_moves, self.generate_moves())
    }

    fn execute(&mut self, action: &Self::Action, player: Self::Player) -> Self::Fitness {
        self.apply_move_unchecked(action);
        match self.result {
            GameResult::FirstPlayerWon => {
                if player {
                    i32::MAX
                } else {
                    i32::MIN
                }
            }
            GameResult::SecondPlayerWon => {
                if player {
                    i32::MIN
                } else {
                    i32::MAX
                }
            }
            GameResult::InProgress => 0,
        }
    }

    #[inline]
    fn is_upper_bound(&self, fitness: Self::Fitness, _player: Self::Player) -> bool {
        fitness == i32::MAX
    }

    #[inline]
    fn is_lower_bound(&self, fitness: Self::Fitness, _player: Self::Player) -> bool {
        fitness == i32::MIN
    }
}

pub enum Opponent<T: IntoRunCondition + Clone> {
    Human,
    AI(Bot<Game>, T),
}

impl<T: IntoRunCondition + Clone> Opponent<T> {
    pub fn new_ai(for_first_player: bool, condition: &T) -> Self {
        Opponent::AI(Bot::new(for_first_player), condition.clone())
    }

    pub fn get_move(&mut self, game: &Game) -> GameMove<Coord> {
        match self {
            Opponent::Human => Self::get_human_move(game),
            Opponent::AI(bot, cond) => {
                let mut logger = Logger::new(cond.clone());
                let res = bot.select(&game, &mut logger).unwrap();

                println!(
                    "Info: steps: {}, depth: {}, completed: {}, duration: {:?}",
                    logger.steps(),
                    logger.depth(),
                    logger.completed(),
                    logger.duration()
                );
                res
            }
        }
    }

    fn get_human_move(game: &Game) -> GameMove<Coord> {
        println!("Your turn, please enter your move: ");
        loop {
            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("Failed to read input");
            let len_without_newline = input.trim_end().len();
            input.truncate(len_without_newline);

            match parsers::parse_move(input.as_str()) {
                Ok(mov) => match game.convert_move_coords(&mov) {
                    Ok(m) => break m,
                    Err(_) => println!("Invalid user coordinate. Please try again: "),
                },
                Err(_) => println!("Can't parse move. Please try again: "),
            }
        }
    }
}

pub fn play_game<T>(mut game: Game, mut player1: Opponent<T>, mut player2: Opponent<T>)
where
    T: IntoRunCondition + Clone,
{
    fn step<T>(player: &mut Opponent<T>, game: &mut Game) -> GameMove<Coord>
    where
        T: IntoRunCondition + Clone,
    {
        loop {
            let mov = player.get_move(&game);
            match game.apply_move(&mov) {
                Ok(_) => break mov,
                Err(msg) => println!("Can't apply move: {}", msg),
            }
        }
    }

    while !game.is_game_over() {
        let who_moves = if game.player1_moves { &mut player1 } else { &mut player2 };
        let mov = step(who_moves, &mut game);
        println!("Move: {}", mov);
        println!("{}", game);
    }

    println!(
        "Game over, {} player won",
        if game.result == GameResult::FirstPlayerWon {
            "first"
        } else {
            "second"
        }
    );
}
