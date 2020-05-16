use crate::ai::AlphaBetaAI;
use crate::coord::Coord;
use crate::game::{Game, GameMove};
use crate::parsers;
use crate::play::Strategy;

use std::io;

pub struct Human;

impl Strategy for Human {
    fn get_move(&mut self, game: &Game) -> GameMove<Coord> {
        println!("\n");
        println!("Current game state: {}", game);
        println!("\n");
        println!("Move syntax: place W1 at R2C2, move B3 from R1C1 to R1C2, fight at R2C3");
        println!("Your turn, please enter your move.");
        loop {
            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("Failed to read input");

            if input.trim_end().starts_with("hint") || input.trim_end().starts_with("dur") {
                let parts: Vec<_> = input.trim_end().split_whitespace().collect();
                let mut dur = 10;
                if parts.len() > 1 {
                    dur = parts[1].parse().unwrap();
                }
                let mut ai = AlphaBetaAI::with_duration(game.player1_moves, dur);
                let m = ai.get_move(game);
                println!("AI recommends: {}", game.userify_move(&m));
            } else if input.trim_end().starts_with("depth") {
                let parts: Vec<_> = input.trim_end().split_whitespace().collect();
                let mut depth = 5;
                if parts.len() > 1 {
                    depth = parts[1].parse().unwrap();
                }
                let mut ai = AlphaBetaAI::with_depth(game.player1_moves, depth);
                let m = ai.get_move(game);
                println!("AI recommends: {}", game.userify_move(&m));
            } else if input.trim_end().starts_with("comp") {
                let mut ai = AlphaBetaAI::to_completion(game.player1_moves);
                let m = ai.get_move(game);
                println!("AI recommends: {}", game.userify_move(&m));
            } else {
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
}
