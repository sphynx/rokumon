use failure::{bail, format_err, Fallible};
use std::fmt;
use std::str::FromStr;

use itertools::Itertools;
use std::collections::{BTreeMap, BTreeSet};

use crate::card::{Card, Deck, Die};
use crate::coord::{Coord, UserCoord};
use crate::game::GameMove;

#[cfg(feature = "with_serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A type of grid to used for the game. Most of the game scenarios
/// use hex grid (also known as "bricks layout"), but the basic one
/// uses square grid.
#[cfg_attr(feature = "with_serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Grid {
    Hex,
    Square,
}

#[cfg_attr(feature = "with_serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Layout {
    /// 3 x 2 layout using square grid. Act 1 in The Rules.
    Rectangle6,
    /// 4 + 3 layout using hex grid, as in Act 2, 3 and 4.
    Bricks7,
    /// Hex layout for Automa.
    Hex7,
    /// Any custom layout.
    #[allow(unused)]
    Custom(Grid, BTreeSet<Coord>),
}

impl FromStr for Layout {
    type Err = failure::Error;
    fn from_str(s: &str) -> Fallible<Self> {
        match s {
            "rectangle6" | "r6" => Ok(Layout::Rectangle6),
            "bricks7" | "b7" => Ok(Layout::Bricks7),
            "hex7" | "h7" => Ok(Layout::Hex7),
            // TODO: support parsing of custom layouts if needed
            _ => bail!("can't parse layout: {}", s),
        }
    }
}

type Cards = BTreeMap<Coord, Card>;

/// Represents the whole game board: cards at particular positions and
/// a type of grid used (to make sense of positions).
#[cfg_attr(feature = "with_serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Board {
    pub grid: Grid,
    #[cfg_attr(feature = "with_serde", serde(with = "serde_cards"))]
    pub cards: Cards,
    pub layout: Layout,
    adj_triples: Vec<(Coord, Coord, Coord)>,
}

#[cfg(feature = "with_serde")]
mod serde_cards {
    use super::*;

    pub fn serialize<S>(cards: &Cards, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.collect_seq(cards)
    }

    pub fn deserialize<'de, D>(d: D) -> Result<Cards, D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec = <Vec<(Coord, Card)>>::deserialize(d)?;
        let map = vec.into_iter().collect();
        Ok(map)
    }
}

impl Board {
    pub fn new(layout: Layout, deck: Deck) -> Self {
        let mut cards_at_positions = BTreeMap::new();
        let mut cards = deck.into_iter();
        let grid;

        match layout.clone() {
            Layout::Bricks7 => {
                grid = Grid::Hex;
                // I use (0, 0, 0) coordinate for bottom-left
                // position. Two rows: top row has y = -1, bottom row
                // has y = 0.

                /*
                      +----+----+----+
                x     |  1 |  2 |  3 |
                y     | -1 | -1 | -1 |
                z     |  0 | -1 | -2 |
                   +--+-+--+-+--+-+--+-+
                x  |  0 |  1 |  2 |  3 |
                y  |  0 |  0 |  0 |  0 |
                z  |  0 | -1 | -2 | -3 |
                   +----+----+----+----+
                */
                for y in -1..=0 {
                    for x in -y..4 {
                        // 4 cards in the bottom row, 3 cards in the top row.
                        cards_at_positions.insert(
                            Coord::new_hex(x, y),
                            cards.next().expect("Board::new: not enough cards in deck"),
                        );
                    }
                }
                assert!(cards.next().is_none(), "Board::new: some cards left in the deck");
            }
            Layout::Rectangle6 => {
                /*
                  +----+----+----+
                x |  0 |  1 |  2 |
                y | -1 | -1 | -1 |
                z |  0 |  0 |  0 |
                  +----+----+----+
                x |  0 |  1 |  2 |
                y |  0 |  0 |  0 |
                z |  0 |  0 |  0 |
                  +----+----+----
                */
                grid = Grid::Square;
                // Again, (0, 0, 0) is bottom-left corner. We have two
                // rows with y = -1 (top) and y = 0 (bottom)
                for y in -1..=0 {
                    for x in 0..3 {
                        cards_at_positions.insert(
                            Coord::new_square(x, y),
                            cards.next().expect("Board::new: not enough cards in deck"),
                        );
                    }
                }
                assert!(cards.next().is_none(), "Board::new: some cards left in the deck");
            }
            Layout::Hex7 => {
                /*
                     +----+----+
                x    |  1 |  2 |
                y    | -1 | -1 |
                z    |  0 | -1 |                          -
                  +--+-+--+-+--+-+
                x |  0 |  1 |  2 |
                y |  0 |  0 |  0 |
                z |  0 | -1 | -2 |
                  +--+-+--+-+--+-+
                x    |  0 |  1 |
                y    |  1 |  1 |
                z    | -1 | -2 |
                     +----+----+
                */
                grid = Grid::Hex;

                let coords = [(1, -1), (2, -1), (0, 0), (1, 0), (2, 0), (0, 1), (1, 1)];

                for (x, y) in &coords {
                    cards_at_positions.insert(
                        Coord::new_hex(*x, *y),
                        cards.next().expect("Board::new: not enough cards in deck"),
                    );
                }

                assert!(cards.next().is_none(), "Board::new: some cards left in the deck");
            }
            Layout::Custom(g, coords) => {
                grid = g;
                for &c in coords.iter() {
                    cards_at_positions.insert(c, cards.next().expect("Board::new: not enough cards in deck"));
                }
                assert!(cards.next().is_none(), "Board::new: some cards left in the deck");
            }
        }

        let adj_triples = Self::adjacent_triples(&grid, cards_at_positions.keys());

        Self {
            grid,
            layout,
            cards: cards_at_positions,
            adj_triples,
        }
    }

    pub fn new_coord(&self, x: i8, y: i8) -> Coord {
        match self.grid {
            Grid::Hex => Coord::new_hex(x, y),
            Grid::Square => Coord::new_square(x, y),
        }
    }

    pub fn refresh_adj_triples(&mut self) {
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
    pub fn bounding_box(&self) -> (i8, i8, i8, i8) {
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
    pub fn row_cards_iter(&self, row: i8) -> impl Iterator<Item = &Card> {
        self.row_iter(row).map(|(_, card)| card)
    }

    /// Iterator over all cards (in arbitrary order).
    pub fn cards_iter(&self) -> impl Iterator<Item = &Card> {
        self.cards.values()
    }

    /// Iterator over all cards along with their coordinates (in
    /// arbitrary order).
    #[allow(unused)]
    pub fn coord_cards_iter(&self) -> impl Iterator<Item = (&Coord, &Card)> {
        self.cards.iter()
    }

    /// Iterator over all coordinates (in arbitrary order).
    pub fn coords_iter(&self) -> impl Iterator<Item = &Coord> {
        self.cards.keys()
    }

    /// Positions of cards from given `row` (`y` coordinate) ordered
    /// by `x` coordinate.
    pub fn row_positions_iter(&self, row: i8) -> impl Iterator<Item = &Coord> {
        self.row_iter(row).map(|(pos, _)| pos)
    }

    /// Iterator over positions/cards from given `row` (`y`
    /// coordinate) ordered by `x` coordinate.
    pub fn row_iter(&self, row: i8) -> impl Iterator<Item = (&Coord, &Card)> {
        self.cards
            .iter()
            .filter(move |(coord, _)| coord.y == row)
            .sorted_by_key(|(coord, _)| coord.x)
    }

    /// Iterator over positions/cards with no dice on them.
    pub fn empty_cards_iter(&self) -> impl Iterator<Item = (&Coord, &Card)> {
        self.cards.iter().filter(|(_, card)| card.is_empty())
    }

    /// Check is the position has a card and it's empty. Note that "no
    /// card" is different to "empty card".
    pub fn has_empty_card_at(&self, coord: &Coord) -> bool {
        self.card_at(coord).map(|c| c.is_empty()).unwrap_or(false)
    }

    /// A card at particular coordinate.
    pub fn card_at(&self, coord: &Coord) -> Option<&Card> {
        self.cards.get(coord)
    }

    /// A mutable references to a card at a particular coordinate.
    pub fn card_at_mut(&mut self, coord: &Coord) -> Option<&mut Card> {
        self.cards.get_mut(coord)
    }

    /// Iterates over active dice (i.e. dice on cards which are not
    /// covered by other dice).
    pub fn active_dice_iter(&self, for_player1: bool) -> impl Iterator<Item = (&Coord, &Die)> {
        self.cards.iter().filter_map(move |(coord, card)| {
            card.top_die().and_then(move |d| {
                if d.belongs_to_player1() == for_player1 {
                    Some((coord, d))
                } else {
                    None
                }
            })
        })
    }

    /// Iterator over neighbouring (immediately adjacent) cards for a
    /// give position excluding given `exclude` coord.
    pub fn neighbours_iter_without(&self, pos: Coord, exclude: Coord) -> impl Iterator<Item = &Coord> {
        self.cards
            .keys()
            .filter(move |c| Self::distance(&self.grid, c, &pos) == 1 && *c != &exclude)
    }

    /// Convert from user coordinates to internal coordinates in a GameMove.
    pub fn convert_move_coords(&self, m: &GameMove<UserCoord>) -> Fallible<GameMove<Coord>> {
        fn go(board: &Board, uc: &UserCoord) -> Fallible<Coord> {
            board.convert_coordinates(uc)
        }

        // Sadly, there are no functors in Rust so we have to apply
        // a function to "holes" in GameMove by hand.
        use GameMove::*;
        Ok(match m {
            Place(d, uc) => Place(d.clone(), go(self, uc)?),
            Move(d, uc_from, uc_to) => Move(d.clone(), go(self, uc_from)?, go(self, uc_to)?),
            Fight(uc) => Fight(go(self, uc)?),
            Surprise(uc_from, to) => Surprise(go(self, uc_from)?, *to),
            Submit => Submit,
        })
    }

    /// Convert from internal coordinates to user coordinates in a GameMove.
    pub fn convert_move_coords_to_user(&self, m: &GameMove<Coord>) -> GameMove<UserCoord> {
        fn go(board: &Board, c: &Coord) -> UserCoord {
            board.convert_coordinates_to_user(c)
        }

        // Sadly, there are no functors in Rust so we have to apply
        // a function to "holes" in GameMove by hand.
        use GameMove::*;
        match m {
            Place(d, uc) => Place(d.clone(), go(self, uc)),
            Move(d, uc_from, uc_to) => Move(d.clone(), go(self, uc_from), go(self, uc_to)),
            Fight(uc) => Fight(go(self, uc)),
            Surprise(uc_from, to) => Surprise(go(self, uc_from), *to),
            Submit => Submit,
        }
    }

    /// Checks if three positions are adjacent to each other.
    pub fn are_three_adjacent(grid: &Grid, x: &Coord, y: &Coord, z: &Coord) -> bool {
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
    pub fn are_three_in_line(grid: &Grid, a: &Coord, b: &Coord, c: &Coord) -> bool {
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

    pub fn num_of_adjacent_triples(&self, c: Coord) -> usize {
        self.adj_triples
            .iter()
            .map(|t| if c == t.0 || c == t.1 || c == t.2 { 1 } else { 0 })
            .sum()
    }

    /// Manhattan distance on hex and square grids.
    pub fn distance(grid: &Grid, a: &Coord, b: &Coord) -> usize {
        match grid {
            Grid::Hex => (a.x - b.x).abs().max((a.y - b.y).abs()).max((a.z - b.z).abs()) as usize,
            Grid::Square => ((a.x - b.x).abs() + (a.y - b.y).abs()) as usize,
        }
    }

    /// Convert between user visible (row + card) and internal
    /// (hex/square) coordinates in presence of given board.
    pub fn convert_coordinates(&self, user_coord: &UserCoord) -> Fallible<Coord> {
        if user_coord.row == 0 || user_coord.card == 0 {
            bail!("User coord numeration starts from 1");
        }

        let user_row = user_coord.row as usize - 1;
        let user_card = user_coord.card as usize - 1;

        let mut rows: Vec<i8> = self.cards.keys().map(|c| c.y).collect();
        rows.as_mut_slice().sort();
        rows.dedup();

        if user_row <= rows.len() {
            self.row_positions_iter(rows[user_row])
                .nth(user_card)
                .copied()
                .ok_or_else(|| format_err!("Card is out of bounds: {}", user_coord.card))
        } else {
            bail!("Row is out of bounds: {}", user_coord.row);
        }
    }

    /// Convert internal coordinates (hex/square) to user visible coordinates (row
    /// + card) in presence of given board.
    pub fn convert_coordinates_to_user(&self, coord: &Coord) -> UserCoord {
        let top_row = self.top_row(); // this is considered R1.
        let user_row = coord.y - top_row + 1;

        let leftmost_card = self.row_positions_iter(coord.y).min_by_key(|c| c.x).unwrap().x; // this is considered C1
        let user_card = coord.x - leftmost_card + 1;

        UserCoord {
            row: user_row as u8,
            card: user_card as u8,
        }
    }

    pub fn adj_triples_iter(&self) -> impl Iterator<Item = &(Coord, Coord, Coord)> {
        self.adj_triples.iter()
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_rectangle() {
        let b = Board::new(Layout::Rectangle6, Deck::six_shuffled());
        let mut it = b.cards.keys();
        assert_eq!(Some(&Coord::new_square(0, -1)), it.next());
        assert_eq!(Some(&Coord::new_square(0, 0)), it.next());
        assert_eq!(Some(&Coord::new_square(1, -1)), it.next());
        assert_eq!(Some(&Coord::new_square(1, 0)), it.next());
        assert_eq!(Some(&Coord::new_square(2, -1)), it.next());
        assert_eq!(Some(&Coord::new_square(2, 0)), it.next());
        assert_eq!(None, it.next());
    }

    #[test]
    fn test_bricks() {
        let b = Board::new(Layout::Bricks7, Deck::seven_shuffled());
        let mut it = b.cards.keys();
        assert_eq!(Some(&Coord::new_hex(0, 0)), it.next());
        assert_eq!(Some(&Coord::new_hex(1, -1)), it.next());
        assert_eq!(Some(&Coord::new_hex(1, 0)), it.next());
        assert_eq!(Some(&Coord::new_hex(2, -1)), it.next());
        assert_eq!(Some(&Coord::new_hex(2, 0)), it.next());
        assert_eq!(Some(&Coord::new_hex(3, -1)), it.next());
        assert_eq!(Some(&Coord::new_hex(3, 0)), it.next());
        assert_eq!(None, it.next());
    }

    #[test]
    fn test_hex() {
        let b = Board::new(Layout::Hex7, Deck::seven_shuffled());
        let mut it = b.cards.keys();
        assert_eq!(Some(&Coord::new_hex(0, 0)), it.next());
        assert_eq!(Some(&Coord::new_hex(0, 1)), it.next());
        assert_eq!(Some(&Coord::new_hex(1, -1)), it.next());
        assert_eq!(Some(&Coord::new_hex(1, 0)), it.next());
        assert_eq!(Some(&Coord::new_hex(1, 1)), it.next());
        assert_eq!(Some(&Coord::new_hex(2, -1)), it.next());
        assert_eq!(Some(&Coord::new_hex(2, 0)), it.next());
        assert_eq!(None, it.next());
    }
}
