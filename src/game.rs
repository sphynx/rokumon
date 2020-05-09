use rand::seq::SliceRandom;
use std::collections::BTreeSet;
use std::fmt;
use std::str::FromStr;

use failure::{bail, ensure, format_err, Fallible};

use crate::board::{Board, Layout};
use crate::card::{Deck, DiceColor, Die};
use crate::coord::{Coord, UserCoord};
use crate::parsers;

/// A player. Has a name and a stock of dice.
#[derive(Debug, Clone)]
pub struct Player {
    name: String,
    dice: Vec<Die>,
}

impl Player {
    fn first(rules: &Rules) -> Self {
        use DiceColor::*;
        let d = Die::new;

        let dice = if rules.enable_fight_move {
            vec![d(Red, 2), d(Red, 2), d(Red, 4), d(Red, 6)]
        } else {
            // No difference then, just generate 4 equal dice.
            vec![d(Red, 2); 4]
        };

        Player {
            name: String::from("Player 1"),
            dice,
        }
    }

    fn second(rules: &Rules) -> Self {
        use DiceColor::*;
        let d = Die::new;
        let dice = if rules.enable_fight_move {
            vec![d(Black, 1), d(Black, 3), d(Black, 3), d(Black, 5), d(White, 1)]
        } else {
            // No difference then, just generate 5 equal dice.
            vec![d(Black, 1); 5]
        };

        Player {
            name: String::from("Player 2"),
            dice,
        }
    }

    fn remove_die(&mut self, die: &Die) -> Fallible<()> {
        let die_ix = self
            .dice
            .iter()
            .position(|d| d == die)
            .ok_or_else(|| format_err!("remove_die: no die {} found in player's stock", &die))?;
        self.dice.remove(die_ix);
        Ok(())
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let dice_str: &str = &self
            .dice
            .iter()
            .map(|d| format!("{}", d))
            .collect::<Vec<String>>()
            .as_slice()
            .join(", ");

        write!(f, "{}: [{}]", self.name, dice_str)
    }
}

/// Representation of a possible game move. Parametrised by a type of
/// coordinates used (user coordinates or internal ones).
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash)]
pub enum GameMove<C> {
    Place(Die, C),
    Move(Die, C, C),
    Fight(C),
    Surprise(C, Coord),
    Submit,
}

impl FromStr for GameMove<UserCoord> {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parsers::parse_move(s)
    }
}

impl fmt::Display for GameMove<Coord> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use GameMove::*;
        match self {
            Place(die, coord) => write!(f, "place {} at {}", die, coord),
            Move(die, from, to) => write!(f, "move {} from {} to {}", die, from, to),
            Fight(coord) => write!(f, "fight at {}", coord),
            Surprise(from, to) => write!(f, "surprise from {} to {}", from, to),
            Submit => write!(f, "submit"),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum ZIndex {
    Top,
    Bottom,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct FightResult {
    pub losing_die: Die,
    pub losing_position: ZIndex,
}

/// Current game result, can be either won by one of the player or
/// still in progress.
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum GameResult {
    InProgress,
    FirstPlayerWon,
    SecondPlayerWon,
}

/// Variations in game rules. Currently, it's whether we allow certain
/// moves or not.
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub struct Rules {
    enable_fight_move: bool,
    enable_surprise_move: bool,
}

impl Default for Rules {
    fn default() -> Self {
        Rules {
            enable_fight_move: true,
            enable_surprise_move: true,
        }
    }
}

impl Rules {
    pub fn new(enable_fight_move: bool, enable_surprise_move: bool) -> Self {
        Rules {
            enable_fight_move,
            enable_surprise_move,
        }
    }
}

/// Represents the whole game state with board, players and additional
/// state variables (whose move it is, number of used "surprises" and
/// the game result).
#[derive(Debug, Clone)]
pub struct Game {
    pub board: Board,
    rules: Rules,
    player1: Player,
    player2: Player,
    pub player1_moves: bool,
    player1_surprises: u8,
    player2_surprises: u8,
    pub result: GameResult,
    pub history: Vec<GameMove<Coord>>,
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Board:")?;
        write!(f, "{}", self.board)?;
        writeln!(f, "{}", self.player1)?;
        writeln!(f, "{}", self.player2)?;
        writeln!(
            f,
            "to move: {}",
            if self.player1_moves { "Player 1" } else { "Player 2" }
        )
    }
}

impl Game {
    /// Create a new game (with position from Act 4 for now).
    pub fn new(layout: Layout, deck: Deck, rules: Rules) -> Self {
        Game {
            board: Board::new(layout, deck),
            rules,
            player1: Player::first(&rules),
            player2: Player::second(&rules),
            player1_moves: true,
            player1_surprises: 0,
            player2_surprises: 0,
            result: GameResult::InProgress,
            history: vec![],
        }
    }

    fn current_player(&self) -> &Player {
        if self.player1_moves {
            &self.player1
        } else {
            &self.player2
        }
    }

    fn current_player_mut(&mut self) -> &mut Player {
        if self.player1_moves {
            &mut self.player1
        } else {
            &mut self.player2
        }
    }

    fn current_player_surprises(&self) -> u8 {
        if self.player1_moves {
            self.player1_surprises
        } else {
            self.player2_surprises
        }
    }

    /// Validates a given move in this particular game state, gives an
    /// explanation if not valid, otherwise returns ().
    fn validate_move(&self, game_move: &GameMove<Coord>) -> Fallible<()> {
        use GameMove::*;

        ensure!(
            self.result == GameResult::InProgress,
            "can't apply move to finished game: {:?}",
            self
        );

        match game_move {
            Place(die, coord) => {
                ensure!(
                    self.current_player().dice.contains(die),
                    "place: player doesn't have this die: {}",
                    die
                );

                ensure!(
                    self.board.has_empty_card_at(coord),
                    "place: target card should be empty: {}",
                    coord
                );

                Ok(())
            }
            Move(die, from, to) => {
                // 1. the `die` is on the card `from` and is not covered.
                // 2. card `to` is different kind to card `from`
                // 3. if card `to` has two dice they must be of current players' color
                match (self.board.card_at(from), self.board.card_at(to)) {
                    (Some(card), Some(target_card)) => {
                        let not_covered = card.top_die().map(|d| *d == *die).unwrap_or(true);
                        ensure!(not_covered, "move: the dice to be moved should not be covered");

                        ensure!(
                            card.kind != target_card.kind,
                            "move: `from` and `to` cards should be of different kinds, but they are both {:?}",
                            card.kind
                        );

                        let covers_stack_ok = target_card.dice.len() < 2
                            || target_card
                                .dice
                                .iter()
                                .all(|d| d.belongs_to_player1() == self.player1_moves);
                        ensure!(covers_stack_ok, "move: can only cover one die or two of your own dice");

                        Ok(())
                    }
                    _ => bail!("move: there should be cards at both `from` and `to` locations"),
                }
            }
            Fight(coord) => {
                ensure!(
                    self.rules.enable_fight_move,
                    "fight: fight moves are disabled in the rules"
                );
                match self.board.card_at(coord) {
                    Some(card) => {
                        let num_of_dice = card.dice.len();
                        ensure!(
                            num_of_dice == 2,
                            "fight: can only fight at a card with two dice, but have: {}",
                            num_of_dice
                        );

                        let at_least_one_yours = card.dice.iter().any(|d| d.belongs_to_player1() == self.player1_moves);
                        ensure!(at_least_one_yours, "fight: at least once die should be yours");

                        Ok(())
                    }
                    _ => bail!("fight: there should be a card at: {}", coord),
                }
            }
            Surprise(from, to) => {
                ensure!(
                    self.rules.enable_surprise_move,
                    "surprise: surprise moves are disabled in the rules"
                );
                ensure!(
                    self.board.card_at(from).is_some(),
                    "surprise: should have a card at: {}",
                    from
                );
                ensure!(
                    self.board.card_at(to).is_none(),
                    "surprise: should be empty position at: {}",
                    to
                );

                let neighbours = self.board.neighbours_iter_without(*to, *from).count();
                ensure!(
                    neighbours >= 2,
                    "surprise: new position for the card should have enough neighbours, has: {}",
                    neighbours
                );

                let number_of_surprises = if self.player1_moves {
                    self.player1_surprises
                } else {
                    self.player2_surprises
                };
                ensure!(
                    number_of_surprises < 1,
                    "surprise: player shouldn't use too many surprises, used: {}",
                    number_of_surprises
                );

                Ok(())
            }
            Submit => Ok(()),
        }
    }

    /// Applies a move to the current game state.
    pub fn apply_move(&mut self, game_move: &GameMove<Coord>) -> Fallible<Option<FightResult>> {
        self.validate_move(game_move)?;
        Ok(self.apply_move_unchecked(game_move))
    }

    /// Applies a move to the current game state (without validation).
    pub fn apply_move_unchecked(&mut self, game_move: &GameMove<Coord>) -> Option<FightResult> {
        use GameMove::*;

        let mut fight_result = None;

        // Here we consider all moves validated, so we use unwrap
        // freely, even though there might be still be failures
        // due to programming errors.
        match game_move {
            Place(die, coord) => {
                let player = self.current_player_mut();
                player.remove_die(&die).unwrap();

                // Add the die to the card.
                let card = self.board.card_at_mut(coord).unwrap();
                card.dice.push(die.clone());

                self.result = self.result();
            }

            Move(_die, from, to) => {
                let from_card = self.board.card_at_mut(from).unwrap();
                let die = from_card.dice.pop().unwrap();

                // Here, we should check for victory condition
                // immediately while the die is in flight. If it
                // leads to the win for the other player, we do
                // not continue with the rest of the move.
                let intermediate_result = self.result();
                if intermediate_result != GameResult::InProgress {
                    // Set the game result.
                    self.result = intermediate_result;

                    // Continue with the rest of the move (this won't
                    // affect game result).
                    let to_card = self.board.card_at_mut(to).unwrap();
                    to_card.dice.push(die);
                } else {
                    // Finish the second part of the move and then
                    // update the result.
                    let to_card = self.board.card_at_mut(to).unwrap();
                    to_card.dice.push(die);
                    self.result = self.result();
                }
            }

            Fight(place) => {
                let battle_card = self.board.card_at_mut(place).unwrap();
                let top_die = battle_card.dice.pop().unwrap();
                let bottom_die = battle_card.dice.pop().unwrap();

                let (winner, loser, swapped) = Die::compare_dice(top_die, bottom_die);
                battle_card.dice.push(winner);

                let losing_position = if swapped { ZIndex::Top } else { ZIndex::Bottom };
                let losing_die = loser.clone();
                fight_result = Some(FightResult {
                    losing_die,
                    losing_position,
                });

                if loser.belongs_to_player1() {
                    self.player1.dice.push(loser);
                } else {
                    self.player2.dice.push(loser);
                }

                self.result = self.result();
            }

            Surprise(from, to) => {
                let card = self.board.cards.remove(&from).unwrap();
                self.board.cards.insert(*to, card);
                self.board.refresh_adj_triples();

                if self.player1_moves {
                    self.player1_surprises += 1;
                } else {
                    self.player2_surprises += 1;
                }

                self.result = self.result();
            }

            Submit => {
                if self.player1_moves {
                    self.result = GameResult::SecondPlayerWon;
                } else {
                    self.result = GameResult::FirstPlayerWon;
                }
            }
        };

        self.player1_moves = !self.player1_moves;
        self.history.push(game_move.clone());

        fight_result
    }

    pub fn undo_move(&mut self, game_move: &GameMove<Coord>, fight_result: Option<FightResult>) {
        use GameMove::*;

        self.history.pop();
        self.player1_moves = !self.player1_moves;

        // Here we consider the move which was made validated, so we
        // use unwrap freely, even though there might be still be
        // failures due to programming errors.
        match game_move {
            Place(_die, coord) => {
                // Remove the die from the card.
                let card = self.board.card_at_mut(coord).unwrap();
                let die = card.dice.pop().unwrap();

                // Add the die back to player's stock. Note that it
                // stock, but it doesn't matter from the game
                // perspective.
                // may end up in a different position in the player
                let player = self.current_player_mut();
                player.dice.push(die);

                self.result = self.result();
            }

            Move(_die, from, to) => {
                let to_card = self.board.card_at_mut(to).unwrap();
                let die = to_card.dice.pop().unwrap();

                let from_card = self.board.card_at_mut(from).unwrap();
                from_card.dice.push(die);

                self.result = self.result();
            }

            Fight(place) => {
                let fight_result = fight_result.expect("fight_result should be provided for undo");
                let insertion_index = match fight_result.losing_position {
                    ZIndex::Top => 1,
                    ZIndex::Bottom => 0,
                };

                let die = fight_result.losing_die;
                if die.belongs_to_player1() {
                    self.player1.remove_die(&die).unwrap();
                } else {
                    self.player2.remove_die(&die).unwrap();
                }

                let battle_card = self.board.card_at_mut(place).unwrap();
                battle_card.dice.insert(insertion_index, die);

                self.result = self.result();
            }

            Surprise(from, to) => {
                let card = self.board.cards.remove(&to).unwrap();
                self.board.cards.insert(*from, card);
                self.board.refresh_adj_triples();

                if self.player1_moves {
                    self.player1_surprises -= 1;
                } else {
                    self.player2_surprises -= 1;
                }

                self.result = self.result();
            }

            Submit => {
                self.result = GameResult::InProgress;
            }
        };
    }

    /// Caclulates the game result of a particular game state by
    /// checking two victory conditions ("three-in-a-stack" and
    /// "three-in-a-row").
    fn result(&self) -> GameResult {
        // "Three in a Stack" condition.
        for card in self.board.cards_iter() {
            if card.dice.len() > 2 {
                if card.dice[0].belongs_to_player1() {
                    return GameResult::FirstPlayerWon;
                } else {
                    return GameResult::SecondPlayerWon;
                }
            }
        }

        // "Three in a Row" condition.
        for tri in self.board.adj_triples_iter() {
            let die1 = self.board.card_at(&tri.0).and_then(|c| c.top_die());
            let die2 = self.board.card_at(&tri.1).and_then(|c| c.top_die());
            let die3 = self.board.card_at(&tri.2).and_then(|c| c.top_die());

            match (die1, die2, die3) {
                (Some(die1), Some(die2), Some(die3)) => {
                    if die1.belongs_to_player1() && die2.belongs_to_player1() && die3.belongs_to_player1() {
                        return GameResult::FirstPlayerWon;
                    } else if !die1.belongs_to_player1() && !die2.belongs_to_player1() && !die3.belongs_to_player1() {
                        return GameResult::SecondPlayerWon;
                    } else {
                        continue;
                    }
                }
                _ => continue,
            }
        }

        GameResult::InProgress
    }

    #[allow(unused)]
    pub fn apply_move_str(&mut self, move_str: &str) -> Fallible<Option<FightResult>> {
        let user_move = move_str.parse()?;
        let converted_move = self.convert_move_coords(&user_move)?;
        self.apply_move(&converted_move)
    }

    /// Convert move coordinates from `UserCoord` to `Coord`.
    pub fn convert_move_coords(&self, m: &GameMove<UserCoord>) -> Fallible<GameMove<Coord>> {
        self.board.convert_move_coords(m)
    }

    /// Returns a random move (uniform distribution).
    pub fn random_move(&self) -> GameMove<Coord> {
        let moves = self.generate_moves();
        let mut rng = rand::thread_rng();
        moves.choose(&mut rng).unwrap().clone()
    }

    // Note: The move generator is supposed to be fast, but now I'm
    // generating moves in a rather naive way. This is an obvious
    // candidate for optimization, if we need any.
    /// Generate all legal moves for current game position.
    pub fn generate_moves(&self) -> Vec<GameMove<Coord>> {
        if self.result != GameResult::InProgress {
            return vec![];
        }

        let mut moves = Vec::with_capacity(32);

        let dice: BTreeSet<Die> = self.current_player().dice.iter().cloned().collect();
        for (&coord, _card) in self.board.empty_cards_iter() {
            for d in dice.iter() {
                moves.push(GameMove::Place(d.clone(), coord));
            }
        }

        let active_dice = self.board.active_dice_iter(self.player1_moves);
        let all_positions: Vec<_> = self.board.cards.keys().collect();
        for (from, die) in active_dice {
            for to in &all_positions {
                let candidate = GameMove::Move(die.clone(), *from, **to);
                if self.validate_move(&candidate).is_ok() {
                    moves.push(candidate);
                }
            }
        }

        if self.rules.enable_fight_move {
            for (&pos, _) in self.board.cards.iter().filter(|(_, card)| card.dice.len() > 1) {
                let candidate = GameMove::Fight(pos);
                if self.validate_move(&candidate).is_ok() {
                    moves.push(candidate);
                }
            }
        }

        if self.rules.enable_surprise_move {
            if self.current_player_surprises() == 0 {
                let (left, right, top, bottom) = self.board.bounding_box();
                for &from in self.board.cards.keys() {
                    for x in left - 1..=right + 1 {
                        for y in top - 1..=bottom + 1 {
                            let candidate = GameMove::Surprise(from, self.board.new_coord(x, y));
                            if self.validate_move(&candidate).is_ok() {
                                moves.push(candidate);
                            }
                        }
                    }
                }
            }
        }

        // We don't include Submit as a candidate move ;)
        moves
    }

    /// Returns if the game is over.
    pub fn is_game_over(&self) -> bool {
        self.result != GameResult::InProgress
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use failure::Fallible;

    #[test]
    fn test_validity_of_place() -> Fallible<()> {
        // TODO: this test is somewhat limited since now we are only
        // testing moves valid in the starting position. Extend it
        // when we have `apply_move` (test placing on non-empty
        // cards).
        use GameMove::*;

        let game = Game::new(Layout::Bricks7, Deck::seven_shuffled(), Default::default());

        use DiceColor::*;
        let d = Die::new;
        let c = Coord::new_hex;

        // Positive test cases.
        game.validate_move(&Place(d(Red, 2), c(0, 0)))?;
        game.validate_move(&Place(d(Red, 4), c(0, 0)))?;
        game.validate_move(&Place(d(Red, 6), c(0, 0)))?;
        game.validate_move(&Place(d(Red, 6), c(1, 0)))?;
        game.validate_move(&Place(d(Red, 6), c(2, -1)))?;

        // No dice like this in Player 1's stock.
        assert!(game.validate_move(&Place(d(Red, 5), c(0, 0))).is_err());
        assert!(game.validate_move(&Place(d(Black, 1), c(0, 0))).is_err());

        // No card there.
        assert!(game.validate_move(&Place(d(Red, 6), c(-1, 0))).is_err());

        Ok(())
    }

    #[test]
    fn test_validity_of_surprise() -> Fallible<()> {
        // TODO: this test is somewhat limited since now we are only
        // testing moves valid in the starting position. Extend it
        // when we have `apply_move`: test surprising twice or
        // repositioning the card after another reposition.

        use GameMove::*;

        let game = Game::new(Layout::Bricks7, Deck::seven_shuffled(), Default::default());
        let c = Coord::new_hex;

        // Positive test cases.
        game.validate_move(&Surprise(c(0, 0), c(1, 1)))?;
        game.validate_move(&Surprise(c(0, 0), c(2, 1)))?;
        game.validate_move(&Surprise(c(0, 0), c(2, -2)))?;
        game.validate_move(&Surprise(c(0, 0), c(4, -1)))?;
        game.validate_move(&Surprise(c(0, 0), c(3, -2)))?;

        game.validate_move(&Surprise(c(1, 0), c(0, -1)))?;
        game.validate_move(&Surprise(c(1, 0), c(2, 1)))?;
        game.validate_move(&Surprise(c(1, 0), c(2, -2)))?;
        game.validate_move(&Surprise(c(1, 0), c(4, -1)))?;
        game.validate_move(&Surprise(c(1, 0), c(3, -2)))?;

        game.validate_move(&Surprise(c(1, -1), c(0, 1)))?;
        game.validate_move(&Surprise(c(1, -1), c(1, 1)))?;
        game.validate_move(&Surprise(c(1, -1), c(2, 1)))?;
        game.validate_move(&Surprise(c(1, -1), c(4, -1)))?;
        game.validate_move(&Surprise(c(1, -1), c(3, -2)))?;

        game.validate_move(&Surprise(c(2, -1), c(0, -1)))?;
        game.validate_move(&Surprise(c(2, -1), c(0, 1)))?;
        game.validate_move(&Surprise(c(2, -1), c(1, 1)))?;
        game.validate_move(&Surprise(c(2, -1), c(2, 1)))?;
        game.validate_move(&Surprise(c(2, -1), c(4, -1)))?;

        // Can't reposition to the same spot.
        assert!(game.validate_move(&Surprise(c(0, 0), c(0, 0))).is_err());
        assert!(game.validate_move(&Surprise(c(1, 0), c(1, 0))).is_err());

        // Not enough neighbours.
        assert!(game.validate_move(&Surprise(c(0, 0), c(0, 1))).is_err());
        assert!(game.validate_move(&Surprise(c(0, 0), c(0, -1))).is_err());
        assert!(game.validate_move(&Surprise(c(1, 0), c(1, 1))).is_err());
        assert!(game.validate_move(&Surprise(c(1, 0), c(0, 1))).is_err());
        assert!(game.validate_move(&Surprise(c(1, -1), c(0, -1))).is_err());
        assert!(game.validate_move(&Surprise(c(1, -1), c(2, -2))).is_err());
        assert!(game.validate_move(&Surprise(c(2, -1), c(3, -2))).is_err());
        assert!(game.validate_move(&Surprise(c(2, -1), c(2, -2))).is_err());

        // Can't start from empty spot.
        assert!(game.validate_move(&Surprise(c(3, 3), c(0, 1))).is_err());

        Ok(())
    }

    #[test]
    fn test_apply_move() -> Fallible<()> {
        let c = Coord::new_hex;

        // Submit leads to loss.
        let mut game = Game::new(Layout::Bricks7, Deck::seven_shuffled(), Default::default());

        assert_eq!(game.result, GameResult::InProgress);
        game.apply_move_str("submit")?;
        assert_eq!(game.result, GameResult::SecondPlayerWon);

        // Submit leads to loss #2.
        let mut game = Game::new(Layout::Bricks7, Deck::seven_shuffled(), Default::default());
        game.apply_move_str("place r2 at r1c1")?;
        game.apply_move_str("submit")?;
        assert_eq!(game.result, GameResult::FirstPlayerWon);

        let layout = Layout::Bricks7;
        let deck = Deck::ordered("gggjjjj")?;

        // Fight reduces number of dice on cards.
        let mut game = Game::new(layout.clone(), deck.clone(), Default::default());
        game.apply_move_str("place r2 at r1c1")?;
        game.apply_move_str("place b1 at r2c1")?;
        game.apply_move_str("move r2 from r1c1 to r2c1")?;
        game.apply_move_str("fight at r2c1")?;
        let card = game.board.card_at(&c(0, 0)).unwrap();
        assert_eq!(card.dice.len(), 1);
        assert_eq!(card.dice[0].value, 2);

        // Fight reduces number of dice on cards.
        // White 1 beats red 6.
        let mut game = Game::new(layout.clone(), deck.clone(), Default::default());
        game.apply_move_str("place r6 at r1c1")?;
        game.apply_move_str("place w1 at r2c1")?;
        game.apply_move_str("move r6 from r1c1 to r2c1")?;
        game.apply_move_str("fight at r2c1")?;
        let card = game.board.card_at(&c(0, 0)).unwrap();
        assert_eq!(card.dice.len(), 1);
        assert_eq!(card.dice[0].value, 1);

        // Simplest 3-in-a-row win.
        let mut game = Game::new(layout.clone(), deck.clone(), Default::default());
        game.apply_move_str("place r6 at r1c1")?;
        game.apply_move_str("place w1 at r2c1")?;
        game.apply_move_str("place r4 at r1c2")?;
        game.apply_move_str("place b3 at r2c2")?;
        game.apply_move_str("place r2 at r1c3")?;
        assert_eq!(game.result, GameResult::FirstPlayerWon);

        // 3-in-a-row doesn't count if it's adjacent.
        let mut game = Game::new(layout.clone(), deck.clone(), Default::default());
        game.apply_move_str("place r6 at r2c1")?;
        game.apply_move_str("place w1 at r1c1")?;
        game.apply_move_str("place r4 at r2c2")?;
        game.apply_move_str("place b3 at r1c2")?;
        game.apply_move_str("place r2 at r2c4")?;
        assert_eq!(game.result, GameResult::InProgress);

        // 3-in-a-row for newly built line.
        let mut game = Game::new(layout.clone(), deck.clone(), Default::default());
        game.apply_move_str("place r6 at r1c1")?;
        game.apply_move_str("place w1 at r2c2")?;
        game.apply_move_str("place r4 at r2c1")?;
        game.apply_move_str("surprise from r2c4 to <2,-2,0>")?;
        game.apply_move_str("place r2 at r1c1")?;
        assert_eq!(game.result, GameResult::FirstPlayerWon);

        // Uncovering a die with 3-in-a-row leads to win.
        let mut game = Game::new(layout.clone(), deck.clone(), Default::default());
        game.apply_move_str("place r6 at r2c1")?;
        game.apply_move_str("place w1 at r1c1")?;
        game.apply_move_str("place r4 at r2c2")?;
        game.apply_move_str("move w1 from r1c1 to r2c2")?;
        game.apply_move_str("place r2 at r2c3")?;
        assert_eq!(game.result, GameResult::InProgress);
        game.apply_move_str("move w1 from r2c2 to r1c1")?;
        assert_eq!(game.result, GameResult::FirstPlayerWon);

        // No move between the same card kinds is allowed.
        let mut game = Game::new(layout.clone(), deck.clone(), Default::default());
        game.apply_move_str("place r6 at r2c1")?;
        game.apply_move_str("place w1 at r1c1")?;
        assert!(game.apply_move_str("move r6 from r2c1 to r2c2").is_err());

        // Simplest 3-in-a-stack win.
        let mut game = Game::new(layout.clone(), deck.clone(), Default::default());
        game.apply_move_str("place r6 at r2c1")?;
        game.apply_move_str("place w1 at r2c2")?;
        game.apply_move_str("place r4 at r1c1")?;
        game.apply_move_str("place b1 at r1c2")?;
        game.apply_move_str("move r4 from r1c1 to r2c1")?;
        game.apply_move_str("move b1 from r1c2 to r2c2")?;
        game.apply_move_str("place r2 at r1c1")?;
        game.apply_move_str("place b3 at r1c2")?;
        assert_eq!(game.result, GameResult::InProgress);
        game.apply_move_str("move r2 from r1c1 to r2c1")?;
        assert_eq!(game.result, GameResult::FirstPlayerWon);

        // Simultaneous 3-in-a-row for both players.
        // (plus a stack and actually 4-in-a-row)

        let deck = Deck::ordered("gggjjjg")?;
        let mut game = Game::new(layout, deck, Default::default());
        game.apply_move_str("place r2 at r2c1")?;
        game.apply_move_str("place b1 at r1c1")?;
        game.apply_move_str("place r2 at r2c2")?;
        game.apply_move_str("move b1 from r1c1 to r2c2")?;
        game.apply_move_str("place r4 at r2c3")?;
        game.apply_move_str("place b3 at r1c3")?;
        game.apply_move_str("place r6 at r2c4")?;
        game.apply_move_str("place b3 at r1c1")?;
        game.apply_move_str("move r6 from r2c4 to r2c3")?;
        assert_eq!(game.result, GameResult::InProgress);
        game.apply_move_str("move b1 from r2c2 to r1c2")?;
        assert_eq!(game.result, GameResult::FirstPlayerWon);

        Ok(())
    }

    #[test]
    pub fn test_move_gen() -> Fallible<()> {
        let deck = Deck::ordered("gggjjjj")?;
        let mut game = Game::new(Layout::Bricks7, deck, Default::default());
        assert_eq!(game.generate_moves().len(), 56);
        game.apply_move_str("place r2 at r2c1")?;
        assert_eq!(game.generate_moves().len(), 59);
        game.apply_move_str("place b1 at r1c1")?;
        assert_eq!(game.generate_moves().len(), 53);
        Ok(())
    }

    #[test]
    pub fn test_applied_moved_to_finished_game_bug() -> Fallible<()> {
        let deck = Deck::ordered("gggjjjj")?;
        let mut game = Game::new(Layout::Bricks7, deck, Default::default());
        game.apply_move_str("place r2 at r2c3")?;
        game.apply_move_str("place b3 at r2c1")?;
        game.apply_move_str("place r6 at r2c4")?;
        game.apply_move_str("place b5 at r1c2")?;
        game.apply_move_str("place r4 at r1c3")?;
        game.apply_move_str("move b5 from r1c2 to r2c1")?;
        game.apply_move_str("move r4 from r1c3 to r2c2")?;
        assert_eq!(game.result, GameResult::FirstPlayerWon);

        Ok(())
    }
}
