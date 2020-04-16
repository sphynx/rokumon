use std::collections::{HashMap, HashSet};
use std::fmt;
use std::str::FromStr;

use failure::{bail, ensure, format_err, Fallible};
use itertools::Itertools;
use rand::seq::SliceRandom;

use crate::parsers;

/// Possible colors of dice.
#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
pub enum DiceColor {
    Red,
    Black,
    White,
}

/// Possible card kinds. There are three of them so far.
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum CardKind {
    Jade,
    Gold,
    #[allow(unused)]
    Fort,
}

/// A die in the game. Has a color and value. It's a normal cube die,
/// so values are from 1 to 6.
#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct Die {
    color: DiceColor,
    value: u8,
}

impl Die {
    pub fn new(color: DiceColor, value: u8) -> Self {
        Die { value, color }
    }

    pub fn belongs_to_player1(&self) -> bool {
        self.color == DiceColor::Red
    }

    /// Returns (winner, loser) pair and whether swap happened.
    pub fn compare_dice(d1: Die, d2: Die) -> (Die, Die, bool) {
        if d1.color == DiceColor::White && d2.value == 6 && d2.color == DiceColor::Red {
            (d1, d2, false)
        } else if d2.color == DiceColor::White && d1.value == 6 && d1.color == DiceColor::Red {
            (d2, d1, true)
        } else if d1.value > d2.value {
            (d1, d2, false)
        } else {
            (d2, d1, true)
        }
    }
}

// B3, W1, R6 and so on.
impl fmt::Display for Die {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let color = match &self.color {
            DiceColor::Red => "R",
            DiceColor::White => "W",
            DiceColor::Black => "B",
        };
        write!(f, "{}{}", color, self.value)
    }
}

/// A card in the game. It is of certain kind and may have dice on it.
#[derive(Clone, Debug)]
pub struct Card {
    kind: CardKind,

    /// The dice are appended to the end. I.e. the top-most die which
    /// covers everything is the last in the vector.
    dice: Vec<Die>,
}

// Jade[], Gold[R6 > W1], Jade[B3] and so on.
impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let kind = match &self.kind {
            CardKind::Jade => "Jade",
            CardKind::Gold => "Gold",
            CardKind::Fort => "Fort",
        };

        let dice_str: &str = &self
            .dice
            .iter()
            .map(|d| format!("{}", d))
            .collect::<Vec<String>>()
            .as_slice()
            .join(" < ");

        write!(f, "{}[{}]", kind, dice_str)
    }
}

impl Card {
    fn main_die(&self) -> Option<&Die> {
        self.dice.last()
    }
}

/// Coordinates used for both hex and square grids.
///
/// For square coordinates z is always zero.
/// For hex coordinates there is an invariant: x + y + z = 0.
///
/// There are different possible coordinates for hex grid. Here we use
/// so-called "cube coordinates". They basically originate from a 3D
/// cube sliced by a plane `x + y + z = 0`. They are very convenient
/// for determining if two hexes lie in the same line (then they have
/// equal x, y or z coordinates), which is useful for 3-in-line
/// determination.
///
/// More details about "cube coordinates" and tons of useful
/// hex-related information can be found here:
/// https://www.redblobgames.com/grids/hexagons/#coordinates-cube
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct Coord {
    x: i8,
    y: i8,
    z: i8,
}

impl fmt::Display for Coord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl Coord {
    /// Create a new hex grid coordinate (based on hex "cube
    /// coordinates") as described in documentation for this struct.
    pub fn new_hex(x: i8, y: i8) -> Self {
        Coord { x, y, z: -x - y }
    }

    /// Create a new square grid coordinate as described in
    /// documentation for this struct.
    #[allow(unused)]
    pub fn new_square(x: i8, y: i8) -> Self {
        Coord { x, y, z: 0 }
    }

    /// Convert between user visible (row + card) and internal
    /// (hex/square) coordinates in presence of given board.
    fn from_user_coord(user_coord: &UserCoord, board: &Board) -> Fallible<Self> {
        if user_coord.row == 0 || user_coord.card == 0 {
            bail!("User coord numeration starts from 1");
        }

        let user_row = user_coord.row as usize - 1;
        let user_card = user_coord.card as usize - 1;

        let mut rows: Vec<i8> = board.cards.keys().map(|c| c.y).collect();
        rows.as_mut_slice().sort();
        rows.dedup();

        if user_row <= rows.len() {
            board
                .row_positions_iter(rows[user_row])
                .nth(user_card)
                .copied()
                .ok_or_else(|| format_err!("Card is out of bounds: {}", user_coord.card))
        } else {
            bail!("Row is out of bounds: {}", user_coord.row);
        }
    }
}

/// More user-friendly coordinates easier to undertand and type. Row
/// is counted from 1 and start from the top. Card is counted from 1
/// and starts from the left.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct UserCoord {
    row: u8,
    card: u8,
}

impl FromStr for UserCoord {
    type Err = failure::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        let chars: Vec<char> = trimmed.chars().collect();
        let usage = format!("Can't parse UserCoord, expected four chars like this: R1C2, got: {}", s);

        if chars.len() != 4 {
            bail!(usage);
        } else {
            return match (chars[0].to_ascii_lowercase(), chars[2].to_ascii_lowercase()) {
                ('r', 'c') => Ok(UserCoord {
                    row: trimmed[1..2].parse::<u8>()?,
                    card: trimmed[3..4].parse::<u8>()?,
                }),
                _ => bail!(usage),
            };
        }
    }
}

impl UserCoord {
    pub fn new(row: u8, card: u8) -> Self {
        UserCoord { row, card }
    }
}

/// A type of grid to used for the game. Most of the game scenarios
/// use hex grid (also known as "bricks layout"), but the basic one
/// uses square grid.
#[derive(Copy, Clone, Debug)]
pub enum Grid {
    Hex,

    #[allow(unused)]
    Square,
}

/// Represents the whole game board: cards at particular positions and
/// a type of grid used (to make sense of positions).
#[derive(Debug)]
pub struct Board {
    grid: Grid,
    cards: HashMap<Coord, Card>,
    adj_triples: Vec<(Coord, Coord, Coord)>,
}

impl Board {
    #[allow(unused)]
    pub fn empty() -> Self {
        Self {
            grid: Grid::Hex,
            cards: HashMap::new(),
            adj_triples: vec![],
        }
    }

    pub fn new_coord(&self, x: i8, y: i8) -> Coord {
        match self.grid {
            Grid::Hex => Coord::new_hex(x, y),
            Grid::Square => Coord::new_square(x, y),
        }
    }

    /// Add 7 cards in a standard hex layout, first 3 cards for the
    /// top row, then 4 cards for the bottom row.
    pub fn with_seven_cards(seven_cards: [Card; 7]) -> Self {
        let mut cards = HashMap::new();
        let mut ix = 0;

        // Put them on the board in right places. Coordinates are a
        // little bit tricky, because they are essential hex
        // coordinates. See here for more details:
        // https://www.redblobgames.com/grids/hexagons/implementation.html#shape-triangle
        // I use (0, 0, 0) coordinate for bottom-left position.

        // Two rows: top row has y = -1, bottom row has y = 0.
        for y in -1..=0 {
            for x in -y..4 {
                // 4 cards in the bottom row, 3 cards in the top row.
                cards.insert(Coord::new_hex(x, y), seven_cards[ix].clone());
                ix += 1;
            }
        }

        assert!(cards.len() == 7);

        let grid = Grid::Hex;
        let adj_triples = Self::adjacent_triples(&grid, cards.keys());

        Self {
            grid,
            cards,
            adj_triples,
        }
    }

    /// Starting position of Act 4.
    pub fn starting_position() -> Self {
        let jade = Card {
            kind: CardKind::Jade,
            dice: vec![],
        };
        let gold = Card {
            kind: CardKind::Gold,
            dice: vec![],
        };

        // Init the deck with 4 Jades and 3 Gold cards.
        let mut deck = [
            jade.clone(),
            jade.clone(),
            jade.clone(),
            jade.clone(),
            gold.clone(),
            gold.clone(),
            gold.clone(),
        ];

        // Shuffle.
        let mut rng = rand::thread_rng();
        &mut deck[..].shuffle(&mut rng);

        Board::with_seven_cards(deck)
    }

    fn refresh_adj_triples(&mut self) {
        self.adj_triples = Self::adjacent_triples(&self.grid, self.cards.keys());
    }

    /// The top-most row (i.e. one with the minimal y-coordinate).
    fn top_row(&self) -> i8 {
        self.cards
            .keys()
            .map(|c| c.y)
            .min()
            .expect("top_row: cards should be non-empty")
    }

    /// The bottom-most row (i.e. one with the maximal y-coordinate).
    fn bottom_row(&self) -> i8 {
        self.cards
            .keys()
            .map(|c| c.y)
            .max()
            .expect("bottom_row: cards should be non-empty")
    }

    /// Bounding box (left, right, top, bottom), inclusive.
    fn bounding_box(&self) -> (i8, i8, i8, i8) {
        let mut left = std::i8::MAX;
        let mut right = std::i8::MIN;
        let mut top = std::i8::MAX;
        let mut bottom = std::i8::MIN;

        for c in self.cards.keys() {
            if c.x < left {
                left = c.x;
            }
            if c.x > right {
                right = c.x;
            }
            if c.y > bottom {
                bottom = c.y;
            }
            if c.y < top {
                top = c.y;
            }
        }

        (left, right, top, bottom)
    }

    /// Cards from given `row` (`y` coordinate) ordered by `x`
    /// coordinate.
    fn row_cards_iter(&self, row: i8) -> impl Iterator<Item = &Card> {
        self.row_iter(row).map(|(_, card)| card)
    }

    /// Iterator over all cards in arbitrary order.
    fn cards_iter(&self) -> impl Iterator<Item = &Card> {
        self.cards.values()
    }

    /// Positions of cards from given `row` (`y` coordinate) ordered
    /// by `x` coordinate.
    fn row_positions_iter(&self, row: i8) -> impl Iterator<Item = &Coord> {
        self.row_iter(row).map(|(pos, _)| pos)
    }

    /// Iterator over positions/cards from given `row` (`y`
    /// coordinate) ordered by `x` coordinate.
    fn row_iter(&self, row: i8) -> impl Iterator<Item = (&Coord, &Card)> {
        self.cards
            .iter()
            .filter(move |(coord, _)| coord.y == row)
            .sorted_by_key(|(coord, _)| coord.x)
    }

    /// Iterator over positions/cards with no dice on them.
    fn empty_cards_iter(&self) -> impl Iterator<Item = (&Coord, &Card)> {
        self.cards.iter().filter(|(_, card)| card.dice.is_empty())
    }

    /// Check is the position has a card and it's empty. Note that "no
    /// card" is different to "empty card".
    fn has_empty_card_at(&self, coord: &Coord) -> bool {
        self.card_at(coord).map(|c| c.dice.is_empty()).unwrap_or(false)
    }

    /// A card at particular coordinate.
    fn card_at(&self, coord: &Coord) -> Option<&Card> {
        self.cards.get(coord)
    }

    /// A mutable references to a card at a particular coordinate.
    fn card_at_mut(&mut self, coord: &Coord) -> Option<&mut Card> {
        self.cards.get_mut(coord)
    }

    fn active_dice_iter(&self, player1_moves: bool) -> impl Iterator<Item = (&Coord, &Die)> {
        self.cards.iter().filter_map(move |(coord, card)| {
            card.dice.last().and_then(move |d| {
                if d.belongs_to_player1() == player1_moves {
                    Some((coord, d))
                } else {
                    None
                }
            })
        })
    }

    /// Iterator over neighbouring (immediately adjacent) cards for a
    /// give position excluding given `exclude` coord.
    fn neighbours_iter_without(&self, pos: Coord, exclude: Coord) -> impl Iterator<Item = &Coord> {
        self.cards
            .keys()
            .filter(move |c| Self::distance(&self.grid, c, &pos) == 1 && *c != &exclude)
    }

    /// Convert coordinates in the GameMove (from user coordinates to
    /// internal coordinates).
    fn convert_move_coords(&self, m: &GameMove<UserCoord>) -> Fallible<GameMove<Coord>> {
        fn go(board: &Board, game_move: &UserCoord) -> Fallible<Coord> {
            Coord::from_user_coord(game_move, board)
        }

        // Sadly, there are no functors in Rust so we have to apply
        // `from_user_coord` to "holes" in GameMove by hand.
        use GameMove::*;
        Ok(match m {
            Place(d, uc) => Place(d.clone(), go(self, uc)?),
            Move(d, uc_from, uc_to) => Move(d.clone(), go(self, uc_from)?, go(self, uc_to)?),
            Fight(uc) => Fight(go(self, uc)?),
            Surprise(uc_from, to) => Surprise(go(self, uc_from)?, *to),
            Submit => Submit,
        })
    }

    /// Checks if three positions are adjacent to each other.
    fn are_three_adjacent(grid: &Grid, x: &Coord, y: &Coord, z: &Coord) -> bool {
        let mut ones = 0;
        if Self::distance(grid, x, y) == 1 {
            ones += 1
        }
        if Self::distance(grid, x, z) == 1 {
            ones += 1
        }
        if Self::distance(grid, y, z) == 1 {
            ones += 1
        }
        ones > 1
    }

    /// Checks if three positions are in one line (but not necessary
    /// adjacent).
    fn are_three_in_line(grid: &Grid, a: &Coord, b: &Coord, c: &Coord) -> bool {
        match grid {
            Grid::Hex => (a.x == b.x && b.x == c.x) || (a.y == b.y && b.y == c.y) || (a.z == b.z && b.z == c.z),
            Grid::Square => (a.x == b.x && b.x == c.x) || (a.y == b.y && b.y == c.y),
        }
    }

    /// Calculate all possible adjacent triples in a single line.
    // TODO: add a test for this (for both grids).
    fn adjacent_triples<'a>(grid: &Grid, coords: impl Iterator<Item = &'a Coord>) -> Vec<(Coord, Coord, Coord)> {
        let mut result = vec![];

        // Note: combinations are without repetitions.
        for tri in coords.combinations(3) {
            let are_adj = Self::are_three_adjacent(grid, tri[0], tri[1], tri[2]);
            let are_in_line = Self::are_three_in_line(grid, tri[0], tri[1], tri[2]);
            if are_adj && are_in_line {
                result.push((*tri[0], *tri[1], *tri[2]));
            }
        }

        result
    }

    /// Manhattan distance on hex and square grids.
    fn distance(grid: &Grid, a: &Coord, b: &Coord) -> usize {
        match grid {
            Grid::Hex => (a.x - b.x).abs().max((a.y - b.y).abs()).max((a.z - b.z).abs()) as usize,
            Grid::Square => ((a.x - b.x).abs() + (a.y - b.y).abs()) as usize,
        }
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let top = self.top_row();
        let bottom = self.bottom_row();

        for y in top..=bottom {
            for c in self.row_cards_iter(y) {
                // TODO: add coordinates as well
                // Without them we can't distinguish some cases.
                write!(f, "{} ", c)?;
            }
            writeln!(f, "")?;
        }

        Ok(())
    }
}

/// A player. Has a name and a stock of dice.
#[derive(Debug)]
pub struct Player {
    name: String,
    dice: Vec<Die>,
}

impl Player {
    fn first() -> Self {
        use DiceColor::*;
        let d = Die::new;
        Player {
            name: String::from("Player 1"),
            dice: vec![d(Red, 2), d(Red, 2), d(Red, 4), d(Red, 6)],
        }
    }

    fn second() -> Self {
        use DiceColor::*;
        let d = Die::new;
        Player {
            name: String::from("Player 2"),
            dice: vec![d(Black, 1), d(Black, 3), d(Black, 3), d(Black, 5), d(White, 1)],
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

/// Current game result, can be either won by one of the player or
/// still in progress.
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum GameResult {
    InProgress,
    FirstPlayerWon,
    SecondPlayerWon,
}

/// Represents the whole game state with board, players and additional
/// state variables (whose move it is, number of used "surprises" and
/// the game result).
#[derive(Debug)]
pub struct Game {
    board: Board,
    player1: Player,
    player2: Player,
    player1_moves: bool,
    player1_surprises: u8,
    player2_surprises: u8,
    result: GameResult,
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
    pub fn new() -> Self {
        Game {
            board: Board::starting_position(),
            player1: Player::first(),
            player2: Player::second(),
            player1_moves: true,
            player1_surprises: 0,
            player2_surprises: 0,
            result: GameResult::InProgress,
        }
    }

    /// Create a custom game with predefined set of seven cards.
    /// Layout is standard. First 3 cards go into top row, next 4 go
    /// into the bottom row.
    #[allow(unused)]
    pub fn custom(cards: [Card; 7]) -> Self {
        Game {
            board: Board::with_seven_cards(cards),
            player1: Player::first(),
            player2: Player::second(),
            player1_moves: true,
            player1_surprises: 0,
            player2_surprises: 0,
            result: GameResult::InProgress,
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
                        let not_covered = card.main_die().map(|d| *d == *die).unwrap_or(true);
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
            Fight(coord) => match self.board.card_at(coord) {
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
            },
            Surprise(from, to) => {
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
    fn apply_move(&mut self, game_move: &GameMove<Coord>) -> Fallible<Option<FightResult>> {
        use GameMove::*;

        self.validate_move(game_move)?;

        let fight_result;

        // Here we consider all moves validated, so we use unwrap
        // freely, even though there might be still be failures
        // due to programming errors.
        match game_move {
            Place(die, coord) => {
                let player = self.current_player_mut();
                player.remove_die(&die)?;

                // Add the die to the card.
                let card = self.board.card_at_mut(coord).unwrap();
                card.dice.push(die.clone());

                self.result = self.result();
                fight_result = None;
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
                    self.result = intermediate_result;
                } else {
                    let to_card = self.board.card_at_mut(to).unwrap();
                    to_card.dice.push(die);

                    self.result = self.result();
                }

                fight_result = None;
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
                fight_result = None;
            }

            Submit => {
                if self.player1_moves {
                    self.result = GameResult::SecondPlayerWon;
                } else {
                    self.result = GameResult::FirstPlayerWon;
                }
                fight_result = None;
            }
        };

        self.player1_moves = !self.player1_moves;

        Ok(fight_result)
    }

    fn undo_move(&mut self, game_move: &GameMove<Coord>, fight_result: Option<FightResult>) {
        use GameMove::*;

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
                // may end up in a different position in the player
                // stock, but it doesn't matter from the game
                // perspective.
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
        for tri in self.board.adj_triples.iter() {
            let die1 = self.board.card_at(&tri.0).and_then(|c| c.main_die());
            let die2 = self.board.card_at(&tri.1).and_then(|c| c.main_die());
            let die3 = self.board.card_at(&tri.2).and_then(|c| c.main_die());

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
        let converted_move = self.board.convert_move_coords(&user_move)?;
        self.apply_move(&converted_move)
    }

    // Note: The move generator is supposed to be fast, but now I'm
    // generating moves in a rather naive way. This is an obvious
    // candidate for optimization, if we need any.
    fn generate_moves(&self) -> Vec<GameMove<Coord>> {
        let mut moves: HashSet<GameMove<Coord>> = HashSet::new();

        let stock = &self.current_player().dice;
        for (&coord, _card) in self.board.empty_cards_iter() {
            for d in stock {
                moves.insert(GameMove::Place(d.clone(), coord));
            }
        }

        let active_dice = self.board.active_dice_iter(self.player1_moves);
        let all_positions: Vec<_> = self.board.cards.keys().collect();
        for (from, die) in active_dice {
            for to in &all_positions {
                let candidate = GameMove::Move(die.clone(), *from, **to);
                if self.validate_move(&candidate).is_ok() {
                    moves.insert(candidate);
                }
            }
        }

        for (&pos, _) in self.board.cards.iter().filter(|(_, card)| card.dice.len() > 1) {
            let candidate = GameMove::Fight(pos);
            if self.validate_move(&candidate).is_ok() {
                moves.insert(candidate);
            }
        }

        if self.current_player_surprises() == 0 {
            let (left, right, top, bottom) = self.board.bounding_box();
            for &from in self.board.cards.keys() {
                for x in left - 1..=right + 1 {
                    for y in top - 1..=bottom + 1 {
                        let candidate = GameMove::Surprise(from, self.board.new_coord(x, y));
                        if self.validate_move(&candidate).is_ok() {
                            moves.insert(candidate);
                        }
                    }
                }
            }
        }

        // We don't include Submit as a candidate move ;)

        moves.into_iter().collect()
    }

    pub fn perft(&mut self, depth: usize) -> Fallible<usize> {
        let moves = self.generate_moves();
        if depth == 1 {
            return Ok(moves.len());
        }

        let mut result = 0;
        for m in moves {
            let fight_result = self.apply_move(&m)?;
            result += self.perft(depth - 1)?;
            self.undo_move(&m, fight_result);
        }

        Ok(result)
    }
}

/// Representation of a possible game move. Parametrised by a type of
/// coordinates used (user coordinates or internal ones).
#[allow(unused)]
#[derive(PartialEq, Eq, Debug, Clone, Hash)]
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

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct FightResult {
    pub losing_die: Die,
    pub losing_position: ZIndex,
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum ZIndex {
    Top,
    Bottom,
}

#[cfg(test)]
mod test {
    use super::*;
    use failure::Fallible;

    #[test]
    fn test_coord_conversion() -> Fallible<()> {
        let board = Board::starting_position();

        macro_rules! test {
            ($row:literal, $card:literal => $x:literal, $y:literal, $z:literal) => {
                assert_eq!(
                    Coord::from_user_coord(
                        &UserCoord {
                            row: $row,
                            card: $card
                        },
                        &board
                    )?,
                    Coord {
                        x: $x,
                        y: $y,
                        z: $z
                    }
                );
            };
        }

        macro_rules! test_failure {
            ($row:literal, $card:literal) => {
                assert!(Coord::from_user_coord(
                    &UserCoord {
                        row: $row,
                        card: $card
                    },
                    &board
                )
                .is_err());
            };
        }

        test!(1, 1 => 1, -1, 0);
        test!(1, 2 => 2, -1, -1);
        test!(1, 3 => 3, -1, -2);
        test!(2, 1 => 0, 0, 0);
        test!(2, 2 => 1, 0, -1);
        test!(2, 3 => 2, 0, -2);
        test!(2, 4 => 3, 0, -3);

        test_failure!(0, 1);
        test_failure!(1, 0);
        test_failure!(20, 1);

        Ok(())
    }

    #[test]
    fn test_user_coord_parsing() -> Fallible<()> {
        macro_rules! test {
            ($input:literal => $row:literal, $card:literal) => {
                assert_eq!(
                    $input.parse::<UserCoord>()?,
                    UserCoord {
                        row: $row,
                        card: $card
                    }
                );
            };
        }

        macro_rules! test_failure {
            ($input:literal) => {
                assert!($input.parse::<UserCoord>().is_err());
            };
        }

        test!("R1C2" => 1, 2);
        test!("r4c1" => 4, 1);
        test!("  r2c2  " => 2, 2);

        test_failure!("RR4c1");
        test_failure!("R4 C1");
        test_failure!("R4C111");
        test_failure!("12");

        Ok(())
    }

    #[test]
    fn test_validity_of_place() -> Fallible<()> {
        // TODO: this test is somewhat limited since now we are only
        // testing moves valid in the starting position. Extend it
        // when we have `apply_move` (test placing on non-empty
        // cards).
        use GameMove::*;

        let game = Game::new();

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

        let game = Game::new();
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

    /// Layout is as follows:
    /// ```
    ///  G G G
    /// J J J J
    /// ```
    fn standard_deck() -> [Card; 7] {
        let jade = Card {
            kind: CardKind::Jade,
            dice: vec![],
        };

        let gold = Card {
            kind: CardKind::Gold,
            dice: vec![],
        };

        [
            gold.clone(),
            gold.clone(),
            gold.clone(),
            jade.clone(),
            jade.clone(),
            jade.clone(),
            jade.clone(),
        ]
    }

    #[test]
    fn test_apply_move() -> Fallible<()> {
        let c = Coord::new_hex;

        // Submit leads to loss.
        let mut game = Game::new();
        assert_eq!(game.result, GameResult::InProgress);
        game.apply_move_str("submit")?;
        assert_eq!(game.result, GameResult::SecondPlayerWon);

        // Submit leads to loss #2.
        let mut game = Game::new();
        game.apply_move_str("place r2 at r1c1")?;
        game.apply_move_str("submit")?;
        assert_eq!(game.result, GameResult::FirstPlayerWon);

        let deck = standard_deck();

        // Fight reduces number of dice on cards.
        let mut game = Game::custom(deck.clone());
        game.apply_move_str("place r2 at r1c1")?;
        game.apply_move_str("place b1 at r2c1")?;
        game.apply_move_str("move r2 from r1c1 to r2c1")?;
        game.apply_move_str("fight at r2c1")?;
        let card = game.board.card_at(&c(0, 0)).unwrap();
        assert_eq!(card.dice.len(), 1);
        assert_eq!(card.dice[0].value, 2);

        // Fight reduces number of dice on cards.
        // White 1 beats red 6.
        let mut game = Game::custom(deck.clone());
        game.apply_move_str("place r6 at r1c1")?;
        game.apply_move_str("place w1 at r2c1")?;
        game.apply_move_str("move r6 from r1c1 to r2c1")?;
        game.apply_move_str("fight at r2c1")?;
        let card = game.board.card_at(&c(0, 0)).unwrap();
        assert_eq!(card.dice.len(), 1);
        assert_eq!(card.dice[0].value, 1);

        // Simplest 3-in-a-row win.
        let mut game = Game::custom(deck.clone());
        game.apply_move_str("place r6 at r1c1")?;
        game.apply_move_str("place w1 at r2c1")?;
        game.apply_move_str("place r4 at r1c2")?;
        game.apply_move_str("place b3 at r2c2")?;
        game.apply_move_str("place r2 at r1c3")?;
        assert_eq!(game.result, GameResult::FirstPlayerWon);

        // 3-in-a-row doesn't count if it's adjacent.
        let mut game = Game::custom(deck.clone());
        game.apply_move_str("place r6 at r2c1")?;
        game.apply_move_str("place w1 at r1c1")?;
        game.apply_move_str("place r4 at r2c2")?;
        game.apply_move_str("place b3 at r1c2")?;
        game.apply_move_str("place r2 at r2c4")?;
        assert_eq!(game.result, GameResult::InProgress);

        // 3-in-a-row for newly built line.
        let mut game = Game::custom(deck.clone());
        game.apply_move_str("place r6 at r1c1")?;
        game.apply_move_str("place w1 at r2c2")?;
        game.apply_move_str("place r4 at r2c1")?;
        game.apply_move_str("surprise from r2c4 to <2,-2,0>")?;
        game.apply_move_str("place r2 at r1c1")?;
        assert_eq!(game.result, GameResult::FirstPlayerWon);

        // Uncovering a die with 3-in-a-row leads to win.
        let mut game = Game::custom(deck.clone());
        game.apply_move_str("place r6 at r2c1")?;
        game.apply_move_str("place w1 at r1c1")?;
        game.apply_move_str("place r4 at r2c2")?;
        game.apply_move_str("move w1 from r1c1 to r2c2")?;
        game.apply_move_str("place r2 at r2c3")?;
        assert_eq!(game.result, GameResult::InProgress);
        game.apply_move_str("move w1 from r2c2 to r1c1")?;
        assert_eq!(game.result, GameResult::FirstPlayerWon);

        // No move between the same card kinds is allowed.
        let mut game = Game::custom(deck.clone());
        game.apply_move_str("place r6 at r2c1")?;
        game.apply_move_str("place w1 at r1c1")?;
        assert!(game.apply_move_str("move r6 from r2c1 to r2c2").is_err());

        // Simplest 3-in-a-stack win.
        let mut game = Game::custom(deck.clone());
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
        let jade = Card {
            kind: CardKind::Jade,
            dice: vec![],
        };

        let gold = Card {
            kind: CardKind::Gold,
            dice: vec![],
        };

        let deck = [
            gold.clone(),
            gold.clone(),
            gold.clone(),
            jade.clone(),
            jade.clone(),
            jade.clone(),
            gold.clone(),
        ];

        let mut game = Game::custom(deck.clone());
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
        let deck = standard_deck();
        let mut game = Game::custom(deck.clone());
        assert_eq!(game.generate_moves().len(), 56);
        game.apply_move_str("place r2 at r2c1")?;
        assert_eq!(game.generate_moves().len(), 59);
        game.apply_move_str("place b1 at r1c1")?;
        assert_eq!(game.generate_moves().len(), 53);
        Ok(())
    }
}
