// use crate::card::{DiceColor, Die};
use crate::coord::Coord;
use crate::game::{Game, GameMove, GameResult};
use crate::play::Strategy;

use cfg_if::cfg_if;
use rubot::{self, Bot, Depth, ToCompletion};

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
        evaluate_for_player(self, player)
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

fn evaluate_for_player(game: &Game, player: bool) -> i32 {
    let eval = evaluate_for_first_player(game);
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

fn evaluate_for_first_player(game: &Game) -> i32 {
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

#[cfg(feature = "for_wasm")]
pub mod web_duration {
    use rubot::{IntoRunCondition, RunCondition};
    use std::time::Duration;

    pub struct WebDuration(pub Duration);

    impl IntoRunCondition for WebDuration {
        type RunCondition = WebInstant;

        fn into_run_condition(self) -> WebInstant {
            let window = web_sys::window().expect("should have a window in this context");
            let performance = window.performance().expect("performance should be available");
            let now = performance.now();
            WebInstant(self.0.as_millis() as f64 + now)
        }
    }

    pub struct WebInstant(f64);

    impl RunCondition for WebInstant {
        #[inline]
        fn step(&mut self) -> bool {
            let window = web_sys::window().expect("should have a window in this context");
            let performance = window.performance().expect("performance should be available");
            performance.now() < self.0
        }

        #[inline]
        fn depth(&mut self, _: u32) -> bool {
            let window = web_sys::window().expect("should have a window in this context");
            let performance = window.performance().expect("performance should be available");
            performance.now() < self.0
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
                let action = if cfg!(feature = "for_wasm") {
                    // no logger, since it uses Duration which is not directly supported in WASM
                    self.bot.detailed_select(&game, $condition).unwrap()
                } else {
                    let mut logger = rubot::Logger::new($condition);
                    let action = self
                        .bot
                        .detailed_select(&game, &mut logger)
                        .expect("Bot returned no moves");
                    println!(
                        "AI log: steps: {}, depth: {}, completed: {}, duration: {:?}",
                        logger.steps(),
                        logger.depth(),
                        logger.completed(),
                        logger.duration()
                    );
                    action
                };

                // Evaluation from current player perspective.
                println!("AI evaluation: {}", pp_evaluation(action.fitness));

                // Evaluations in PV are printed from the first player perspective.
                println!("PV:");
                let mut game_tmp = game.clone();
                for (ix, m) in action.path.iter().enumerate() {
                    let um = game_tmp.userify_move(&m);
                    game_tmp.apply_move_unchecked(&m);
                    let score = evaluate_for_first_player(&game_tmp);
                    println!("{}: {}, eval: {}", ix + 1, um, pp_evaluation(score));
                }
                println!();

                return action.path.first().unwrap().clone();
            };
        }

        if self.duration != 0 {
            cfg_if! {
                if #[cfg(feature = "for_wasm")] {
                    let web_duration = web_duration::WebDuration(Duration::from_secs(self.duration));
                    run_ai_until!(web_duration);
                } else {
                    let duration = Duration::from_secs(self.duration);
                    println!("Running AI with duration {:?}...", &duration);
                    run_ai_until!(duration);
                }
            }
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

fn pp_evaluation(score: i32) -> String {
    if score == i32::MAX {
        String::from("inf")
    } else if score == i32::MIN {
        String::from("-inf")
    } else {
        format!("{}", score)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::board::Layout;
    use crate::card::{Deck, DiceColor, Die};
    use crate::coord::UserCoord;
    use crate::game::Rules;
    use failure::Fallible;

    #[test]
    fn test_eval() -> Fallible<()> {
        let mut game = Game::new(Layout::Bricks7, Deck::ordered("JJJGGGG")?, Rules::default());
        assert_eq!(evaluate_for_first_player(&game), 0);
        game.apply_user_move(&GameMove::Place(Die::new(DiceColor::Red, 2), UserCoord::new(1, 2)))?;
        assert_eq!(evaluate_for_first_player(&game), 1);
        game.apply_user_move(&GameMove::Place(Die::new(DiceColor::Black, 3), UserCoord::new(2, 2)))?;
        assert_eq!(evaluate_for_first_player(&game), -1);

        Ok(())
    }
}
