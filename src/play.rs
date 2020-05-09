use crate::coord::Coord;
use crate::game::{Game, GameMove, GameResult};
use crate::strategy::Strategy;

pub fn play_game(mut game: Game, mut player1: impl Strategy, mut player2: impl Strategy) {
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

        println!("\n\nPlayed move: {}\n\n", mov);
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
}
