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

pub enum Opponent<T: IntoRunCondition + Clone> {
    Human,
    Random,
    AI(Bot<Game>, T),
}

impl<T: IntoRunCondition + Clone> Opponent<T> {
    pub fn new_ai(for_first_player: bool, condition: &T) -> Self {
        Opponent::AI(Bot::new(for_first_player), condition.clone())
    }

    pub fn get_move(&mut self, game: &Game) -> GameMove<Coord> {
        match self {
            Opponent::Human => Self::get_human_move(game),
            Opponent::Random => game.random_move(),
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
        "Game over, {} player won, {} moves",
        if game.result == GameResult::FirstPlayerWon {
            "first"
        } else {
            "second"
        },
        game.history.len()
    );
}

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
