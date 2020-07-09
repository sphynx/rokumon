use failure::{bail, Fallible};
use rand::seq::SliceRandom;
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

#[cfg(feature = "with_serde")]
use serde::{Deserialize, Serialize};

/// Possible colors of dice.
#[cfg_attr(feature = "with_serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug, Hash)]
pub enum DiceColor {
    Red,
    Black,
    White,
}

/// A die in the game. Has a color and value. It's a normal cube die,
/// so values are from 1 to 6.
#[cfg_attr(feature = "with_serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Hash)]
pub struct Die {
    pub color: DiceColor,
    pub value: u8,
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
            DiceColor::Red => "r",
            DiceColor::White => "w",
            DiceColor::Black => "b",
        };
        write!(f, "{}{}", color, self.value)
    }
}

/// Possible card kinds. There are three of them so far.
#[cfg_attr(feature = "with_serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone, Debug)]
pub enum CardKind {
    Jade,
    Gold,
    #[allow(unused)]
    Fort,
}

impl TryFrom<char> for CardKind {
    type Error = failure::Error;
    fn try_from(c: char) -> Fallible<Self> {
        match c {
            'g' | 'G' | 'w' | 'W' => Ok(CardKind::Gold),
            'j' | 'J' | 'b' | 'B' => Ok(CardKind::Jade),
            'f' | 'F' => Ok(CardKind::Fort),
            _ => bail!("unrecognized card: {}", c),
        }
    }
}

/// A card in the game. It is of certain kind and may have dice on it.
#[cfg_attr(feature = "with_serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Debug)]
pub struct Card {
    pub kind: CardKind,

    /// The dice are appended to the end. I.e. the top-most die which
    /// covers everything is the last in the vector.
    pub dice: Vec<Die>,
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
    pub fn top_die(&self) -> Option<&Die> {
        self.dice.last()
    }

    pub fn is_empty(&self) -> bool {
        self.dice.is_empty()
    }
}

impl TryFrom<char> for Card {
    type Error = failure::Error;
    fn try_from(c: char) -> Fallible<Self> {
        let kind = CardKind::try_from(c)?;
        Ok(Card { kind, dice: vec![] })
    }
}

/// Deck is a collection of cards.
#[cfg_attr(feature = "with_serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Debug, Clone)]
pub struct Deck {
    cards: Vec<Card>,
}

impl IntoIterator for Deck {
    type Item = Card;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.cards.into_iter()
    }
}

impl FromStr for Deck {
    type Err = failure::Error;
    fn from_str(s: &str) -> Fallible<Self> {
        let cards: Fallible<Vec<Card>> = s.chars().map(Card::try_from).collect();
        Ok(Deck { cards: cards? })
    }
}

/// Represents a deck of cards to be used in the game.
impl Deck {
    /// A deck with cards in order. Cards are specified by a string
    /// like 'JJJGGGG' for 3 Jade cards and 4 Gold cards. Thy will be
    /// dealt from the top to the bottom, from left to right do the
    /// card positions in the layout.
    pub fn ordered(descr: &str) -> Fallible<Self> {
        descr.parse()
    }

    /// Shuffled deck defined by a specification like 'JJJGGGG'.
    #[allow(unused)]
    pub fn shuffled(descr: &str) -> Fallible<Self> {
        let mut deck: Deck = descr.parse()?;
        deck.shuffle();
        Ok(deck)
    }

    /// A standard deck with 4 Jades and 3 Gold cards which are
    /// randomly shuffled.
    pub fn seven_shuffled() -> Self {
        let mut deck = Deck::ordered("jjjjggg").unwrap();
        deck.shuffle();
        deck
    }

    /// A deck with 6 cards selected out of 4 Jade and 3 Gold cards.
    pub fn six_shuffled() -> Self {
        let mut deck = Deck::ordered("jjjjggg").unwrap();
        deck.shuffle();
        deck.drop_one_card();
        deck
    }

    fn shuffle(&mut self) {
        let mut rng = rand::thread_rng();
        self.cards.as_mut_slice().shuffle(&mut rng);
    }

    fn drop_one_card(&mut self) {
        self.cards.truncate(self.cards.len() - 1);
    }
}
