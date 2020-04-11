use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use failure::{bail, format_err, Fallible};
use itertools::Itertools;
use rand::seq::SliceRandom;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
enum DiceColor {
    Red,
    Black,
    White,
}

#[allow(dead_code)]
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
enum CardKind {
    Jade,
    Gold,
    Fort,
}

#[derive(PartialEq, Eq, Clone, Debug)]
struct Die {
    value: u8,
    color: DiceColor,
}

impl Die {
    pub fn new(value: u8, color: DiceColor) -> Self {
        Die { value, color }
    }

    pub fn belongs_to_player1(&self) -> bool {
        self.color == DiceColor::Red
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

#[derive(Clone, Debug)]
struct Card {
    kind: CardKind,
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
            .join(" > ");

        write!(f, "{}[{}]", kind, dice_str)
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
struct Coord {
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
    /// Convert between two types of coordinates in presence of given
    /// board.
    #[allow(dead_code)]
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

    fn distance(&self, other: &Coord) -> usize {
        (self.x - other.x)
            .abs()
            .max((self.y - other.y).abs())
            .max((self.z - other.z).abs()) as usize
    }
}

/// More user-friendly coordinates easier to undertand and type. Row
/// is counted from 1 and start from the top. Card is counted from 1
/// and starts from the left.
#[derive(PartialEq, Eq, Debug)]
struct UserCoord {
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

#[allow(dead_code)]
#[derive(Debug)]
enum Layout {
    Bricks,
    Square,
}

#[derive(Debug)]
struct Board {
    layout: Layout,
    cards: HashMap<Coord, Card>,
}

impl Board {
    fn starting_position() -> Board {
        let jade = Card {
            kind: CardKind::Jade,
            dice: vec![],
        };
        let gold = Card {
            kind: CardKind::Gold,
            dice: vec![],
        };

        // Init the deck with 4 Jades and 3 Gold cards.
        let mut deck = vec![
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
        deck.as_mut_slice().shuffle(&mut rng);

        // Put them on the board in right places. Coordinates are a
        // little bit tricky, because they are essential hex
        // coordinates. See here for more details:
        // https://www.redblobgames.com/grids/hexagons/implementation.html#shape-triangle
        // I use (0, 0, 0) coordinate for bottom-left position.
        let mut cards = HashMap::new();
        for y in -1..=0 {
            // Two rows.
            for x in -y..4 {
                // 4 cards in the bottom row, 3 cards in the top row.
                cards.insert(
                    Coord { x, y, z: -x - y },
                    deck.pop().expect("Deck should have enough cards for starting position"),
                );
            }
        }

        assert!(cards.len() == 7);

        Board {
            layout: Layout::Bricks,
            cards,
        }
    }

    fn top_row(&self) -> i8 {
        self.cards
            .keys()
            .map(|c| c.y)
            .min()
            .expect("top_row: cards should be non-empty")
    }

    fn bottom_row(&self) -> i8 {
        self.cards
            .keys()
            .map(|c| c.y)
            .max()
            .expect("bottom_row: cards should be non-empty")
    }

    /// Cards from given `row` (`y` coordinate) ordered by `x`
    /// coordinate.
    fn row_cards_iter(&self, row: i8) -> impl Iterator<Item = &Card> {
        self.row_iter(row).map(|(_, card)| card)
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

    fn has_empty_card_at(&self, coord: &Coord) -> bool {
        self.card_at(coord).map(|c| c.dice.is_empty()).unwrap_or(false)
    }

    /// A card at particular coordinate.
    fn card_at(&self, coord: &Coord) -> Option<&Card> {
        self.cards.get(coord)
    }

    /// Iterator over neighbouring (immediately adjacent) cards for a
    /// give position.
    #[allow(dead_code)]
    fn neighbours_iter(&self, pos: Coord) -> impl Iterator<Item = &Coord> {
        self.cards.keys().filter(move |c| c.distance(&pos) == 1)
    }

    /// Iterator over neighbouring (immediately adjacent) cards for a
    /// give position excluding given `exclude` coord.
    fn neighbours_iter_without(&self, pos: Coord, exclude: Coord) -> impl Iterator<Item = &Coord> {
        self.cards
            .keys()
            .filter(move |c| c.distance(&pos) == 1 && *c != &exclude)
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

#[derive(Debug)]
struct Player {
    name: String,
    dice: Vec<Die>,
}

impl Player {
    fn first() -> Self {
        use DiceColor::*;
        let d = Die::new;
        Player {
            name: String::from("Player 1"),
            dice: vec![d(2, Red), d(2, Red), d(4, Red), d(6, Red)],
        }
    }

    fn second() -> Self {
        use DiceColor::*;
        let d = Die::new;
        Player {
            name: String::from("Player 2"),
            dice: vec![d(1, Black), d(3, Black), d(3, Black), d(5, Black), d(1, White)],
        }
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

#[derive(Debug)]
struct Game {
    board: Board,
    player1: Player,
    player2: Player,
    player1_moves: bool,
    player1_surprises: u8,
    player2_surprises: u8,
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
    fn new() -> Self {
        Game {
            board: Board::starting_position(),
            player1: Player::first(),
            player2: Player::second(),
            player1_moves: true,
            player1_surprises: 0,
            player2_surprises: 0,
        }
    }

    fn current_player(&self) -> &Player {
        if self.player1_moves {
            &self.player1
        } else {
            &self.player2
        }
    }

    #[allow(dead_code)]
    fn is_valid_move(&self, mv: &GameMove) -> bool {
        use GameMove::*;
        match mv {
            Place(die, coord) => {
                let player_has_die = self.current_player().dice.contains(die);
                let target_card_is_empty = self.board.has_empty_card_at(coord);
                player_has_die && target_card_is_empty
            }
            Move(die, from, to) => {
                // 1. the `die` is on the card `from` and is not covered.
                // 2. card `to` is different kind to card `from`
                // 3. if card `to` has two dice they must be of current players' color
                match (self.board.card_at(from), self.board.card_at(to)) {
                    (Some(card), Some(target_card)) => {
                        let not_covered = card.dice.last().map(|d| *d == *die).unwrap_or(true);
                        let different_kind = card.kind != target_card.kind;
                        let covers_stack_ok = target_card.dice.len() < 2
                            || target_card
                                .dice
                                .iter()
                                .all(|d| d.belongs_to_player1() == self.player1_moves);

                        not_covered && different_kind && covers_stack_ok
                    }
                    _ => false,
                }
            }
            Fight(coord) => match self.board.card_at(coord) {
                Some(card) => {
                    let has_two_dice = card.dice.len() == 2;
                    let at_least_one_yours = card.dice.iter().any(|d| d.belongs_to_player1() == self.player1_moves);
                    has_two_dice && at_least_one_yours
                }
                _ => false,
            },
            Surprise(from, to) => {
                let has_card_at_from = self.board.card_at(from).is_some();
                let is_empty_at_to = self.board.card_at(to).is_none();
                let to_has_enough_neighbours = self.board.neighbours_iter_without(*to, *from).count() >= 2;
                let not_too_many_surprises = if self.player1_moves {
                    self.player1_surprises
                } else {
                    self.player2_surprises
                } == 0;
                has_card_at_from && is_empty_at_to && to_has_enough_neighbours && not_too_many_surprises
            }
            Submit => true,
        }
    }

    #[allow(dead_code)]
    fn apply_move(&mut self, _mv: &GameMove) {}
}

#[allow(dead_code)]
enum GameMove {
    Place(Die, Coord),
    Move(Die, Coord, Coord),
    Fight(Coord),
    Surprise(Coord, Coord),
    Submit,
}

fn main() {
    println!("Rokumon, v0.1");

    let game = Game::new();
    println!("{}", game);
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
    fn test_validity_of_place() {
        // TODO: this test is somewhat limited since now we are only
        // testing moves valid in the starting position. Extend it
        // when we have `apply_move` (test placing on non-empty
        // cards).

        let game = Game::new();

        use DiceColor::*;
        let d = Die::new;
        fn c(x: i8, y: i8) -> Coord {
            Coord { x, y, z: -x - y }
        }

        // Positive test cases.
        assert!(game.is_valid_move(&GameMove::Place(d(2, Red), c(0, 0))));
        assert!(game.is_valid_move(&GameMove::Place(d(4, Red), c(0, 0))));
        assert!(game.is_valid_move(&GameMove::Place(d(6, Red), c(0, 0))));
        assert!(game.is_valid_move(&GameMove::Place(d(6, Red), c(1, 0))));
        assert!(game.is_valid_move(&GameMove::Place(d(6, Red), c(2, -1))));

        // No dice like this in Player 1's stock.
        assert!(!game.is_valid_move(&GameMove::Place(d(5, Red), c(0, 0))));
        assert!(!game.is_valid_move(&GameMove::Place(d(1, Black), c(0, 0))));

        // No card there.
        assert!(!game.is_valid_move(&GameMove::Place(d(6, Red), c(-1, 0))));
    }

    #[test]
    fn test_validity_of_surprise() {
        // TODO: this test is somewhat limited since now we are only
        // testing moves valid in the starting position. Extend it
        // when we have `apply_move`: test surprising twice or
        // repositioning the card after another reposition.

        let game = Game::new();

        fn c(x: i8, y: i8) -> Coord {
            Coord { x, y, z: -x - y }
        }

        // Positive test cases.
        assert!(game.is_valid_move(&GameMove::Surprise(c(0, 0), c(1, 1))));
        assert!(game.is_valid_move(&GameMove::Surprise(c(0, 0), c(2, 1))));
        assert!(game.is_valid_move(&GameMove::Surprise(c(0, 0), c(2, -2))));
        assert!(game.is_valid_move(&GameMove::Surprise(c(0, 0), c(4, -1))));
        assert!(game.is_valid_move(&GameMove::Surprise(c(0, 0), c(3, -2))));

        assert!(game.is_valid_move(&GameMove::Surprise(c(1, 0), c(0, -1))));
        assert!(game.is_valid_move(&GameMove::Surprise(c(1, 0), c(2, 1))));
        assert!(game.is_valid_move(&GameMove::Surprise(c(1, 0), c(2, -2))));
        assert!(game.is_valid_move(&GameMove::Surprise(c(1, 0), c(4, -1))));
        assert!(game.is_valid_move(&GameMove::Surprise(c(1, 0), c(3, -2))));

        assert!(game.is_valid_move(&GameMove::Surprise(c(1, -1), c(0, 1))));
        assert!(game.is_valid_move(&GameMove::Surprise(c(1, -1), c(1, 1))));
        assert!(game.is_valid_move(&GameMove::Surprise(c(1, -1), c(2, 1))));
        assert!(game.is_valid_move(&GameMove::Surprise(c(1, -1), c(4, -1))));
        assert!(game.is_valid_move(&GameMove::Surprise(c(1, -1), c(3, -2))));

        assert!(game.is_valid_move(&GameMove::Surprise(c(2, -1), c(0, -1))));
        assert!(game.is_valid_move(&GameMove::Surprise(c(2, -1), c(0, 1))));
        assert!(game.is_valid_move(&GameMove::Surprise(c(2, -1), c(1, 1))));
        assert!(game.is_valid_move(&GameMove::Surprise(c(2, -1), c(2, 1))));
        assert!(game.is_valid_move(&GameMove::Surprise(c(2, -1), c(4, -1))));

        // Can't reposition to the same spot.
        assert!(!game.is_valid_move(&GameMove::Surprise(c(0, 0), c(0, 0))));
        assert!(!game.is_valid_move(&GameMove::Surprise(c(1, 0), c(1, 0))));

        // Not enough neighbours.
        assert!(!game.is_valid_move(&GameMove::Surprise(c(0, 0), c(0, 1))));
        assert!(!game.is_valid_move(&GameMove::Surprise(c(0, 0), c(0, -1))));
        assert!(!game.is_valid_move(&GameMove::Surprise(c(1, 0), c(1, 1))));
        assert!(!game.is_valid_move(&GameMove::Surprise(c(1, 0), c(0, 1))));
        assert!(!game.is_valid_move(&GameMove::Surprise(c(1, -1), c(0, -1))));
        assert!(!game.is_valid_move(&GameMove::Surprise(c(1, -1), c(2, -2))));
        assert!(!game.is_valid_move(&GameMove::Surprise(c(2, -1), c(3, -2))));
        assert!(!game.is_valid_move(&GameMove::Surprise(c(2, -1), c(2, -2))));

        // Can't start from empty spot.
        assert!(!game.is_valid_move(&GameMove::Surprise(c(3, 3), c(0, 1))));
    }
}
