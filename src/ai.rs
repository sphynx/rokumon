use crate::card::{DiceColor, Die};
use crate::coord::Coord;
use crate::game::{Game, GameMove, GameResult};
use crate::strategy::Strategy;

use rubot::{self, Bot, Depth, Logger, ToCompletion};

use std::i32;
use std::time::Duration;

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
        let eval = evaluate(self);
        if player {
            eval
        } else {
            if eval == i32::MAX {
                i32::MIN
            } else if eval == i32::MIN {
                i32::MAX
            } else {
                -eval
            }
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

fn evaluate(game: &Game) -> i32 {
    match game.result {
        GameResult::FirstPlayerWon => std::i32::MAX,
        GameResult::SecondPlayerWon => std::i32::MIN,
        GameResult::InProgress => {
            // Count every uncovered die as much time as it's in
            // triples (i.e. central cards will be counted as 2 and
            // side cards as 1 in a typical Bricks7 layout). Get
            // penalty for covered dice.
            game.board
                .coord_cards_iter()
                .map(|(coord, card)| match card.dice.as_slice() {
                    [d1] => {
                        let triples = game.board.num_of_adjacent_triples(*coord) as i32;
                        if d1.belongs_to_player1() {
                            triples
                        } else {
                            -triples
                        }
                    }
                    [_, d2] => {
                        let triples = game.board.num_of_adjacent_triples(*coord) as i32;
                        if d2.belongs_to_player1() {
                            triples + 1
                        } else {
                            -triples - 1
                        }
                    }
                    _ => 0, // Either empty (doesn't affect the score) or 3 (game over)
                })
                .sum()
        }
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
        macro_rules! run_ai_until {
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

        match game.ply_to_be_played() {
            0 => {
                // We are just starting, let's play a random `place`
                // in better squares.
                let coord = game
                    .board
                    .coords_iter()
                    .map(|coord| (*coord, game.board.num_of_adjacent_triples(*coord)))
                    .max_by_key(|(_coord, n)| *n)
                    .unwrap();

                return GameMove::Place(Die::new(DiceColor::Red, 2), coord.0);
            }
            1 => {
                // We are just starting, let's play a random `place`
                // in better empty squares.
                let coord = game
                    .board
                    .empty_cards_iter()
                    .map(|(coord, _card)| (*coord, game.board.num_of_adjacent_triples(*coord)))
                    .max_by_key(|(_coord, n)| *n)
                    .unwrap();

                return GameMove::Place(Die::new(DiceColor::Black, 5), coord.0);
            }
            _ => {
                // Now just run alpha-beta.
                if self.duration != 0 {
                    let duration = Duration::from_secs(self.duration);
                    println!("Running AI with duration {:?}...", &duration);
                    run_ai_until!(duration);
                } else if self.depth != 0 {
                    let depth = Depth(self.depth);
                    println!("Running AI with depth {:?}...", &depth);
                    run_ai_until!(depth);
                } else {
                    println!("Running AI until completion...");
                    run_ai_until!(ToCompletion);
                }
            }
        }
    }
}

/*
#[cfg(test)]
mod test {
    use super::*;

    use crate::board::Layout;
    use crate::card::Deck;
    use crate::game::Rules;
    use failure::Fallible;

    #[test]
    fn test_eval() -> Fallible<()> {
        let mut game = Game::new(Layout::Bricks7, Deck::ordered("JJJGGGG")?, Rules::default());
        assert_eq!(evaluate(&game), 0);
        game.apply_move_str("place r2 at r1c2")?;
        assert_eq!(evaluate(&game), 1);
        game.apply_move_str("place b3 at r2c2")?;
        assert_eq!(evaluate(&game), -1);

        Ok(())
    }
}

*/
