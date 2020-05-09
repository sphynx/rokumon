use crate::coord::Coord;
use crate::game::{Game, GameMove};
use crate::parsers;

use std::io;

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

