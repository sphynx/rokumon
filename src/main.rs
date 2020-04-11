use std::collections::HashMap;
use std::fmt;

use itertools::Itertools;
use rand::seq::SliceRandom;

#[derive(Copy, Clone, Debug)]
enum DiceColor {
    Red,
    Black,
    White,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
enum CardKind {
    Jade,
    Gold,
    Fort,
}

#[derive(Clone, Debug)]
struct Die {
    value: u8,
    color: DiceColor,
}

impl Die {
    pub fn new(value: u8, color: DiceColor) -> Self {
        Die { value, color }
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

#[derive(PartialEq, Eq, Hash, Debug)]
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
                    deck.pop()
                        .expect("Deck should have enough cards for starting position"),
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
        self.cards.keys().map(|c| c.y).min().unwrap()
    }

    fn bottom_row(&self) -> i8 {
        self.cards.keys().map(|c| c.y).max().unwrap()
    }

    /// Cards from given `row` ordered by `x` coordinate.
    fn cards_in_row(&self, row: i8) -> impl Iterator<Item = &Card> {
        self.cards
            .iter()
            .filter(move |(coord, _)| coord.y == row)
            .sorted_by_key(|(coord, _)| coord.x)
            .map(|(_, card)| card)
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let top = self.top_row();
        let bottom = self.bottom_row();

        for y in top..=bottom {
            for c in self.cards_in_row(y) {
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
            dice: vec![
                d(1, Black),
                d(3, Black),
                d(3, Black),
                d(5, Black),
                d(1, White),
            ],
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
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Board:")?;
        write!(f, "{}", self.board)?;
        writeln!(f, "{}", self.player1)?;
        writeln!(f, "{}", self.player2)?;
        writeln!(f, "to move: {}", if self.player1_moves {"1"} else {"2"})
    }
}

impl Game {
    fn new() -> Self {
        Game {
            board: Board::starting_position(),
            player1: Player::first(),
            player2: Player::second(),
            player1_moves: true,
        }
    }
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
