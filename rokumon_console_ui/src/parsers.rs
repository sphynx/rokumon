/// The module with various nom-based parsers.

/// We are parsing the following format for moves,
/// informally given by example:
///
/// - place W1 at R2C2
/// - move B3 from R1C1 to R1C2
/// - fight at R2C3
/// - surprise from R1C2 to <3, -2, -1>
/// - submit
use std::convert::TryFrom;

use failure::{bail, Fallible};

use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::{char, digit1, one_of, space0, space1};
use nom::combinator::{all_consuming, map, map_opt, map_res, value};
use nom::sequence::{delimited, pair, terminated, tuple};
use nom::IResult;
use nom::{do_parse, tag_no_case};

use rokumon::card::{DiceColor, Die};
use rokumon::coord::{Coord, UserCoord};
use rokumon::game::GameMove;

fn unsigned(i: &str) -> IResult<&str, i8> {
    map_res(digit1, |s: &str| s.parse())(i)
}

fn u8_parser(i: &str) -> IResult<&str, u8> {
    map_res(digit1, |s: &str| s.parse())(i)
}

fn sign(i: &str) -> IResult<&str, i8> {
    alt((value(-1, char('-')), |i| Ok((i, 1))))(i)
}

fn signed(i: &str) -> IResult<&str, i8> {
    map(pair(sign, unsigned), |(s, u)| s * u)(i)
}

fn die_value(i: &str) -> IResult<&str, u8> {
    map_opt(one_of("123456"), |c| c.to_digit(10).and_then(|d| u8::try_from(d).ok()))(i)
}

fn die_color(i: &str) -> IResult<&str, DiceColor> {
    alt((
        value(DiceColor::Black, tag_no_case("B")),
        value(DiceColor::White, tag_no_case("W")),
        value(DiceColor::Red, tag_no_case("R")),
    ))(i)
}

fn die(i: &str) -> IResult<&str, Die> {
    map(pair(die_color, die_value), |(color, value)| Die::new(color, value))(i)
}

fn user_coord(i: &str) -> IResult<&str, UserCoord> {
    do_parse!(
        i,
        tag_no_case!("R") >> row: u8_parser >> tag_no_case!("C") >> card: u8_parser >> (UserCoord::new(row, card))
    )
}

fn coord_sep(i: &str) -> IResult<&str, ()> {
    value((), terminated(char(','), space0))(i)
}

fn coord(i: &str) -> IResult<&str, Coord> {
    map(
        delimited(
            char('<'),
            tuple((signed, coord_sep, signed, coord_sep, signed)),
            char('>'),
        ),
        |(x, _, y, _, _)| Coord::new_hex(x, y),
    )(i)
}

fn place_cmd(i: &str) -> IResult<&str, GameMove<UserCoord>> {
    do_parse!(
        i,
        tag_no_case!("place")
            >> space1
            >> d: die
            >> space1
            >> tag_no_case!("at")
            >> space1
            >> c: user_coord
            >> (GameMove::Place(d, c))
    )
}

fn move_cmd(i: &str) -> IResult<&str, GameMove<UserCoord>> {
    do_parse!(
        i,
        tag_no_case!("move")
            >> space1
            >> d: die
            >> space1
            >> tag_no_case!("from")
            >> space1
            >> from: user_coord
            >> space1
            >> tag_no_case!("to")
            >> space1
            >> to: user_coord
            >> (GameMove::Move(d, from, to))
    )
}

fn fight_cmd(i: &str) -> IResult<&str, GameMove<UserCoord>> {
    do_parse!(
        i,
        tag_no_case!("fight") >> space1 >> tag_no_case!("at") >> space1 >> c: user_coord >> (GameMove::Fight(c))
    )
}

fn surprise_cmd(i: &str) -> IResult<&str, GameMove<UserCoord>> {
    do_parse!(
        i,
        tag_no_case!("surprise")
            >> space1
            >> tag_no_case!("from")
            >> space1
            >> from: user_coord
            >> space1
            >> tag_no_case!("to")
            >> space1
            >> to: coord
            >> (GameMove::Surprise(from, to))
    )
}

fn submit_cmd(i: &str) -> IResult<&str, GameMove<UserCoord>> {
    value(GameMove::Submit, tag_no_case("submit"))(i)
}

fn game_move(i: &str) -> IResult<&str, GameMove<UserCoord>> {
    all_consuming(alt((place_cmd, move_cmd, fight_cmd, surprise_cmd, submit_cmd)))(i)
}

pub fn parse_move(s: &str) -> Fallible<GameMove<UserCoord>> {
    match game_move(s) {
        Ok((_, res)) => return Ok(res),
        Err(_) => bail!("Failed to parse move from '{}'", s),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! test {
        ($input:expr => $parsed:expr) => {
            assert_eq!(Ok(("", $parsed)), $input);
        };
    }

    macro_rules! test_failure {
        ($input:expr) => {
            assert!($input.is_err());
        };
    }

    #[test]
    fn test_unsigned() {
        test!(unsigned("25") => 25);
        test!(unsigned("0") => 0);

        test_failure!(unsigned("-1"));
        test_failure!(unsigned("250"));
    }

    #[test]
    fn test_signed() {
        test!(signed("25") => 25);
        test!(signed("0") => 0);
        test!(signed("-0") => 0);
        test!(signed("-25") => -25);

        test_failure!(signed("250"));
    }

    #[test]
    fn test_die() {
        use DiceColor::*;

        let d = Die::new;

        test!(die("R1") => d(Red, 1));
        test!(die("R3") => d(Red, 3));
        test!(die("W1") => d(White, 1));
        test!(die("b6") => d(Black, 6));

        test_failure!(die("w 1"));
        test_failure!(die("b7"));
        test_failure!(die("r0"));
        test_failure!(die("R0"));
        test_failure!(die(" R1"));
    }

    #[test]
    fn test_move() {
        use DiceColor::*;
        use GameMove::*;

        let d = Die::new;
        let c = Coord::new_hex;
        let uc = UserCoord::new;

        test!(game_move("place R3 at R1C2") => Place(d(Red, 3), uc(1, 2)));
        test!(game_move("move W1 from R1C1 to R2C2") => Move(d(White, 1), uc(1, 1), uc(2, 2)));
        test!(game_move("fight at R2C1") => Fight(uc(2, 1)));
        test!(game_move("fight at r1c3") => Fight(uc(1, 3)));
        test!(game_move("surprise from R2C2 to <0, 1, -1>") => Surprise(uc(2, 2), c(0, 1)));
        test!(game_move("surprise FROM R1C2 TO <0,1, -1>") => Surprise(uc(1, 2), c(0, 1)));
        test!(game_move("submit") => Submit);
        test!(game_move("SUBMIT") => Submit);

        test_failure!(game_move("fight R2C1"));
        test_failure!(game_move("fightat R2C1"));
        test_failure!(game_move("fight at <0,0,0>"));
    }
}
