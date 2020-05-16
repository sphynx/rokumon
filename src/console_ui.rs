use crate::ai::AlphaBetaAI;
use crate::coord::Coord;
use crate::game::{Game, GameMove};
use crate::parsers;
use crate::play::Strategy;

use rustyline::error::ReadlineError;
use rustyline::Editor;

use std::process::exit;

pub struct Human;

impl Strategy for Human {
    fn get_move(&mut self, game: &Game) -> GameMove<Coord> {
        let mut rl = Editor::<()>::new();
        if rl.load_history("history.txt").is_err() {
            println!("No previous history.");
        }

        println!("Your turn, please enter your move. For move syntax type `help` or `?`. For current game position type `pos`.");
        loop {
            match rl.readline("> ") {
                Ok(input) => {
                    let cmd = input.as_str().trim_end();
                    rl.add_history_entry(cmd);

                    if cmd.starts_with("hint") || cmd.starts_with("dur") {
                        let parts: Vec<_> = cmd.split_whitespace().collect();
                        let dur = if parts.len() > 1 { parts[1].parse().unwrap() } else { 10 };
                        let mut ai = AlphaBetaAI::with_duration(game.player1_moves, dur);
                        let m = ai.get_move(game);
                        println!("AI recommends: {}", game.userify_move(&m));
                    } else if cmd.starts_with("dep") {
                        let parts: Vec<_> = cmd.split_whitespace().collect();
                        let depth = if parts.len() > 1 { parts[1].parse().unwrap() } else { 5 };
                        let mut ai = AlphaBetaAI::with_depth(game.player1_moves, depth);
                        let m = ai.get_move(game);
                        println!("AI recommends: {}", game.userify_move(&m));
                    } else if cmd.starts_with("comp") {
                        let mut ai = AlphaBetaAI::to_completion(game.player1_moves);
                        let m = ai.get_move(game);
                        println!("AI recommends: {}", game.userify_move(&m));
                    } else if cmd.starts_with("pos") {
                        println!("{}", game);
                    } else if cmd.starts_with("moves") {
                        println!("Game moves:");
                        if game.history.is_empty() {
                            println!("None so far");
                        } else {
                            for (ix, m) in game.history.iter().enumerate() {
                                println!("{}: {}", ix + 1, game.userify_move(&m));
                            }
                        }
                    } else if cmd.starts_with("help") || cmd.starts_with("?") {
                        println!("Examples of moves you can enter:");
                        println!("  place w1 at r2c2");
                        println!("  move b3 from r1c1 to r1c2");
                        println!("  fight at r2c3");
                        println!();

                        println!("Other useful commands for inspecting current game");
                        println!("pos:         display current game position");
                        println!("moves:       display moves made so far");
                        println!();

                        println!("AI related commands allow to get analysis from AI:");
                        println!("hint:        get a hint by running AI for 10s");
                        println!("duration s:  get a hint by running AI for `s` seconds");
                        println!("depth d:     get a hint by running AI down to `d` moves depth");
                        println!("complete:    run AI until it completes its analysis fully");
                    } else {
                        // Try to parse as a move.
                        match parsers::parse_move(input.trim_end()) {
                            Ok(mov) => match game.convert_move_coords(&mov) {
                                Ok(m) => break m,
                                Err(_) => println!("[ERR] invalid user coordinate."),
                            },
                            Err(msg) => println!("[ERR] {}. Use `help` to see how to enter moves", msg),
                        }
                    }
                    rl.save_history("history.txt").unwrap();
                }

                Err(ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    exit(0);
                }
                Err(ReadlineError::Eof) => {
                    println!("CTRL-D");
                    exit(0);
                }
                Err(err) => {
                    println!("EГГОГ: {:?}", err);
                    exit(1);
                }
            }
        }
    }
}
