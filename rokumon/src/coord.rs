use failure::bail;
use std::fmt;
use std::str::FromStr;

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
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Copy, Clone)]
pub struct Coord {
    pub x: i8,
    pub y: i8,
    pub z: i8,
}

impl fmt::Display for Coord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}, {}, {}>", self.x, self.y, self.z)
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
    pub fn new_square(x: i8, y: i8) -> Self {
        Coord { x, y, z: 0 }
    }
}

/// More user-friendly coordinates easier to undertand and type. Row
/// is counted from 1 and start from the top. Card is counted from 1
/// and starts from the left.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct UserCoord {
    pub row: u8,
    pub card: u8,
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

impl fmt::Display for UserCoord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "r{}c{}", self.row, self.card)
    }
}

impl UserCoord {
    pub fn new(row: u8, card: u8) -> Self {
        UserCoord { row, card }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::board::{Board, Layout};
    use crate::card::Deck;
    use failure::Fallible;

    #[test]
    fn test_coord_conversion() -> Fallible<()> {
        let board = Board::new(Layout::Bricks7, Deck::seven_shuffled());

        macro_rules! test {
            ($row:literal, $card:literal => $x:literal, $y:literal, $z:literal) => {
                assert_eq!(
                    board.convert_coordinates(&UserCoord {
                        row: $row,
                        card: $card
                    })?,
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
                assert!(board
                    .convert_coordinates(&UserCoord {
                        row: $row,
                        card: $card
                    })
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
}
