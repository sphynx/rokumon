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

    println!("Starting position: {}", &game);

    fn step(player: &mut impl Strategy, game: &mut Game) -> GameMove<Coord> {
        loop {
            let mov = player.get_move(&game);
            match game.apply_move(&mov) {
                Ok(_) => break mov,
                Err(msg) => println!("[ERR] Can't apply move: {}", msg),
            }
        }
    }

    while !game.is_game_over() {
        let mov = if game.player1_moves {
            step(&mut player1, &mut game)
        } else {
            step(&mut player2, &mut game)
        };

        println!("Played move: {}", game.userify_move(&mov));
    }

    println!(
        "Game over! {} player won in {} moves.",
        if game.result == GameResult::FirstPlayerWon {
            "First"
        } else {
            "Second"
        },
        game.history.len()
    );

    println!("Moves history:");
    for (ix, m) in game.history.iter().enumerate() {
        println!("{}: {}", ix + 1, game.userify_move(&m));
    }

    game.result == GameResult::FirstPlayerWon
}
