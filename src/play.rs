use crate::coord::Coord;
use crate::game::{Game, GameMove, GameResult};

pub trait Strategy {
    fn get_move(&mut self, game: &Game) -> GameMove<Coord>;
}

pub struct RandomAI;

impl Strategy for RandomAI {
    fn get_move(&mut self, game: &Game) -> GameMove<Coord> {
        game.random_move()
    }
}

pub fn play_game(mut game: Game, mut player1: impl Strategy, mut player2: impl Strategy) -> bool {
    fn step(player: &mut impl Strategy, game: &mut Game) -> GameMove<Coord> {
        loop {
            let mov = player.get_move(&game);
            match game.apply_move(&mov) {
                Ok(_) => break mov,
                Err(msg) => println!("Can't apply move: {}", msg),
            }
        }
    }

    while !game.is_game_over() {
        let mov = if game.player1_moves {
            step(&mut player1, &mut game)
        } else {
            step(&mut player2, &mut game)
        };

        let user_coord_move = game.userify_move(&mov);
        println!("\n\nPlayed move: {}\n\n", user_coord_move);
        println!("Resulting position:\n{}", game);
    }

    println!(
        "Game over, {} player won, {} moves",
        if game.result == GameResult::FirstPlayerWon {
            "first"
        } else {
            "second"
        },
        game.history.len()
    );

    game.result == GameResult::FirstPlayerWon
}
