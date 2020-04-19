use rayon::prelude::*;

use crate::coord::Coord;
use crate::game::{Game, GameMove};

/// This perft uses a clone of the game every time it explores a new
/// move. Hence it makes a lot of allocations and uses a lot of
/// memory, but can be parallelized.
pub fn parallel_perft(game: &Game, depth: usize) -> usize {
    fn go(game: &Game, depth: usize, m: &GameMove<Coord>) -> usize {
        let mut copy_game = (*game).clone();
        copy_game.apply_move_unchecked(m);
        parallel_perft(&copy_game, depth - 1)
    }

    let moves = game.generate_moves();
    if depth == 1 {
        moves.len()
    } else {
        moves.par_iter().map(|m| go(game, depth, m)).sum()
    }
}

/// This perft uses incremental updates, i.e. mutates the game
/// constantly by applying and undoing moves. It does not do much
/// allocations, but can't be easily parallelized because of the
/// mutations.
pub fn perft(game: &mut Game, depth: usize) -> usize {
    let moves = game.generate_moves();
    if depth == 1 {
        return moves.len();
    }

    let mut result = 0;

    for m in moves {
        let fight_result = game.apply_move_unchecked(&m);
        result += perft(game, depth - 1);
        game.undo_move(&m, fight_result);
    }

    result
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::board::Layout;
    use crate::card::Deck;
    use crate::game::{Game, Rules};

    #[test]
    pub fn test_perft() -> failure::Fallible<()> {
        let deck = Deck::ordered("gggjjjj")?;
        let rules = Rules::new(true, true);
        let mut game = Game::new(Layout::Bricks7, deck.clone(), rules);
        assert_eq!(perft(&mut game, 1), 56);

        let rules = Rules::new(true, false);
        let mut game = Game::new(Layout::Bricks7, deck.clone(), rules);
        assert_eq!(perft(&mut game, 1), 21);
        assert_eq!(perft(&mut game, 2), 504);
        assert_eq!(perft(&mut game, 3), 7608);
        assert_eq!(perft(&mut game, 4), 130800);

        Ok(())
    }
}
