use std::collections::HashMap;
use std::convert::TryFrom;
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

    /// Returns (winner, loser) pair.
    pub fn compare_dice(d1: Die, d2: Die) -> (Die, Die) {
        if d1.color == DiceColor::White && d2.value == 6 && d2.color == DiceColor::Red {
            (d1, d2)
        } else if d2.color == DiceColor::White && d1.value == 6 && d1.color == DiceColor::Red {
            (d2, d1)
        } else if d1.value > d2.value {
            (d1, d2)
        } else {
            (d2, d1)
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

#[derive(Clone, Debug)]
struct Card {
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
    #[allow(dead_code)]
    fn new(x: i8, y: i8, z: i8) -> Self {
        Coord { x, y, z }
    }

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
}

/// More user-friendly coordinates easier to undertand and type. Row
/// is counted from 1 and start from the top. Card is counted from 1
/// and starts from the left.
#[derive(PartialEq, Eq, Debug, Clone)]
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
impl UserCoord {
    fn new(row: u8, card: u8) -> Self {
        UserCoord { row, card }
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
enum Grid {
    Hex,
    Square,
}

#[derive(Debug)]
struct Board {
    grid: Grid,
    cards: HashMap<Coord, Card>,
    adj_triples: Vec<(Coord, Coord, Coord)>,
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

        let grid = Grid::Hex;
        let adj_triples = Self::adjacent_triples(&grid, cards.keys());

        Board {
            grid,
            cards,
            adj_triples,
        }
    }

    fn refresh_adj_triples(&mut self) {
        self.adj_triples = Self::adjacent_triples(&self.grid, self.cards.keys());
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

    /// Iterator over neighbouring (immediately adjacent) cards for a
    /// give position excluding given `exclude` coord.
    fn neighbours_iter_without(&self, pos: Coord, exclude: Coord) -> impl Iterator<Item = &Coord> {
        self.cards
            .keys()
            .filter(move |c| Self::distance(&self.grid, c, &pos) == 1 && *c != &exclude)
    }

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

    fn are_three_in_line(grid: &Grid, a: &Coord, b: &Coord, c: &Coord) -> bool {
        match grid {
            Grid::Hex => (a.x == b.x && b.x == c.x) || (a.y == b.y && b.y == c.y) || (a.z == b.z && b.z == c.z),
            Grid::Square => (a.x == b.x && b.x == c.x) || (a.y == b.y && b.y == c.y),
        }
    }

    fn adjacent_triples<'a>(grid: &Grid, coords: impl Iterator<Item = &'a Coord>) -> Vec<(Coord, Coord, Coord)> {
        let mut result = vec![];

        for tri in coords.combinations(3) {
            let are_adj = Self::are_three_adjacent(grid, tri[0], tri[1], tri[2]);
            let are_in_line = Self::are_three_in_line(grid, tri[0], tri[1], tri[2]);
            if are_adj && are_in_line {
                result.push((*tri[0], *tri[1], *tri[2]));
            }
        }

        result
    }

    /// Manhattan's distance on hex and square grids.
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

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
enum GameResult {
    InProgress,
    FirstPlayerWon,
    SecondPlayerWon,
}

#[derive(Debug)]
struct Game {
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
    fn new() -> Self {
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

    fn is_valid_move(&self, game_move: &GameMove<Coord>) -> bool {
        use GameMove::*;
        match game_move {
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
                        let not_covered = card.main_die().map(|d| *d == *die).unwrap_or(true);
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
    fn apply_move(&mut self, game_move: &GameMove<Coord>) -> Fallible<()> {
        use GameMove::*;

        if self.is_valid_move(game_move) {
            // Here we consider all moves validated, so we use unwrap
            // freely, even though there might be still be failures
            // due to programming errors.
            match game_move {
                Place(die, coord) => {
                    let player = self.current_player_mut();

                    // Remove the die from player's stock.
                    let die_ix = player.dice.iter().position(|d| d == die).unwrap();
                    player.dice.remove(die_ix);

                    // Add the die to the card.
                    let card = self.board.card_at_mut(coord).unwrap();
                    card.dice.push(die.clone());
                }

                Move(_die, from, to) => {
                    let from_card = self.board.card_at_mut(from).unwrap();
                    let die = from_card.dice.pop().unwrap();

                    // Here, we should check for victory condition
                    // immediately while the die is in flight. If it
                    // leads to the win for the other player, we
                    // immediately stop and report that.
                    let intermediate_result = self.result();
                    if intermediate_result != GameResult::InProgress {
                        self.result = intermediate_result;
                        self.player1_moves = !self.player1_moves;
                        return Ok(());
                    }

                    let to_card = self.board.card_at_mut(to).unwrap();
                    to_card.dice.push(die);
                }

                Fight(place) => {
                    let battle_card = self.board.card_at_mut(place).unwrap();
                    let die1 = battle_card.dice.pop().unwrap();
                    let die2 = battle_card.dice.pop().unwrap();

                    let (winner, loser) = Die::compare_dice(die1, die2);
                    battle_card.dice.push(winner);

                    if loser.belongs_to_player1() {
                        self.player1.dice.push(loser);
                    } else {
                        self.player2.dice.push(loser);
                    }
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
                }

                Submit => {
                    if self.player1_moves {
                        self.result = GameResult::SecondPlayerWon;
                    } else {
                        self.result = GameResult::FirstPlayerWon;
                    }
                }
            };

            self.result = self.result();
            self.player1_moves = !self.player1_moves;

            Ok(())
        } else {
            bail!("The move is invalid")
        }
    }

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
}

#[derive(PartialEq, Eq, Debug, Clone)]
enum GameMove<C> {
    Place(Die, C),
    Move(Die, C, C),
    Fight(C),
    Surprise(C, Coord),
    Submit,
}

fn main() {
    println!("Rokumon, v0.1");

    let game = Game::new();
    println!("{}", game);
}

/// We are parsing the following format, informally given by example:
/// - place W1 at R2C2
/// - move B3 from R1C1 to R1C2
/// - fight at R2C3
/// - surprise from R1C2 to <3, -2, -1>
/// - submit
#[allow(dead_code)]
mod parsers {
    use super::*;

    use nom::branch::alt;
    use nom::bytes::complete::tag_no_case;
    use nom::character::complete::{char, digit1, one_of, space0, space1};
    use nom::combinator::{map, map_opt, map_res, value};
    use nom::sequence::{delimited, pair, terminated, tuple};
    use nom::IResult;
    use nom::{do_parse, tag_no_case};

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
        map(pair(die_color, die_value), |(color, value)| Die { color, value })(i)
    }

    fn user_coord(i: &str) -> IResult<&str, UserCoord> {
        do_parse!(
            i,
            tag_no_case!("R") >> row: u8_parser >> tag_no_case!("C") >> card: u8_parser >> (UserCoord { row, card })
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
            |(x, _, y, _, z)| Coord { x, y, z },
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
        alt((place_cmd, move_cmd, fight_cmd, surprise_cmd, submit_cmd))(i)
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
            test!(die("R1") => Die { color: DiceColor::Red, value: 1 });
            test!(die("R3") => Die { color: DiceColor::Red, value: 3 });
            test!(die("W1") => Die { color: DiceColor::White, value: 1 });
            test!(die("b6") => Die { color: DiceColor::Black, value: 6 });

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
            let c = Coord::new;
            let uc = UserCoord::new;

            test!(game_move("place R3 at R1C2") => Place(d(3, Red), uc(1, 2)));
            test!(game_move("move W1 from R1C1 to R2C2") => Move(d(1, White), uc(1, 1), uc(2, 2)));
            test!(game_move("fight at R2C1") => Fight(uc(2, 1)));
            test!(game_move("fight at r1c3") => Fight(uc(1, 3)));
            test!(game_move("surprise from R2C2 to <0, 1, -1>") => Surprise(uc(2, 2), c(0, 1, -1)));
            test!(game_move("surprise FROM R1C2 TO <0,1, -1>") => Surprise(uc(1, 2), c(0, 1, -1)));
            test!(game_move("submit") => Submit);
            test!(game_move("SUBMIT") => Submit);

            test_failure!(game_move("fight R2C1"));
            test_failure!(game_move("fightat R2C1"));
            test_failure!(game_move("fight at <0,0,0>"));
        }
    }
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
        use GameMove::*;

        let game = Game::new();

        use DiceColor::*;
        let d = Die::new;
        fn c(x: i8, y: i8) -> Coord {
            Coord { x, y, z: -x - y }
        }

        // Positive test cases.
        assert!(game.is_valid_move(&Place(d(2, Red), c(0, 0))));
        assert!(game.is_valid_move(&Place(d(4, Red), c(0, 0))));
        assert!(game.is_valid_move(&Place(d(6, Red), c(0, 0))));
        assert!(game.is_valid_move(&Place(d(6, Red), c(1, 0))));
        assert!(game.is_valid_move(&Place(d(6, Red), c(2, -1))));

        // No dice like this in Player 1's stock.
        assert!(!game.is_valid_move(&Place(d(5, Red), c(0, 0))));
        assert!(!game.is_valid_move(&Place(d(1, Black), c(0, 0))));

        // No card there.
        assert!(!game.is_valid_move(&Place(d(6, Red), c(-1, 0))));
    }

    #[test]
    fn test_validity_of_surprise() {
        // TODO: this test is somewhat limited since now we are only
        // testing moves valid in the starting position. Extend it
        // when we have `apply_move`: test surprising twice or
        // repositioning the card after another reposition.

        use GameMove::*;

        let game = Game::new();

        fn c(x: i8, y: i8) -> Coord {
            Coord { x, y, z: -x - y }
        }

        // Positive test cases.
        assert!(game.is_valid_move(&Surprise(c(0, 0), c(1, 1))));
        assert!(game.is_valid_move(&Surprise(c(0, 0), c(2, 1))));
        assert!(game.is_valid_move(&Surprise(c(0, 0), c(2, -2))));
        assert!(game.is_valid_move(&Surprise(c(0, 0), c(4, -1))));
        assert!(game.is_valid_move(&Surprise(c(0, 0), c(3, -2))));

        assert!(game.is_valid_move(&Surprise(c(1, 0), c(0, -1))));
        assert!(game.is_valid_move(&Surprise(c(1, 0), c(2, 1))));
        assert!(game.is_valid_move(&Surprise(c(1, 0), c(2, -2))));
        assert!(game.is_valid_move(&Surprise(c(1, 0), c(4, -1))));
        assert!(game.is_valid_move(&Surprise(c(1, 0), c(3, -2))));

        assert!(game.is_valid_move(&Surprise(c(1, -1), c(0, 1))));
        assert!(game.is_valid_move(&Surprise(c(1, -1), c(1, 1))));
        assert!(game.is_valid_move(&Surprise(c(1, -1), c(2, 1))));
        assert!(game.is_valid_move(&Surprise(c(1, -1), c(4, -1))));
        assert!(game.is_valid_move(&Surprise(c(1, -1), c(3, -2))));

        assert!(game.is_valid_move(&Surprise(c(2, -1), c(0, -1))));
        assert!(game.is_valid_move(&Surprise(c(2, -1), c(0, 1))));
        assert!(game.is_valid_move(&Surprise(c(2, -1), c(1, 1))));
        assert!(game.is_valid_move(&Surprise(c(2, -1), c(2, 1))));
        assert!(game.is_valid_move(&Surprise(c(2, -1), c(4, -1))));

        // Can't reposition to the same spot.
        assert!(!game.is_valid_move(&Surprise(c(0, 0), c(0, 0))));
        assert!(!game.is_valid_move(&Surprise(c(1, 0), c(1, 0))));

        // Not enough neighbours.
        assert!(!game.is_valid_move(&Surprise(c(0, 0), c(0, 1))));
        assert!(!game.is_valid_move(&Surprise(c(0, 0), c(0, -1))));
        assert!(!game.is_valid_move(&Surprise(c(1, 0), c(1, 1))));
        assert!(!game.is_valid_move(&Surprise(c(1, 0), c(0, 1))));
        assert!(!game.is_valid_move(&Surprise(c(1, -1), c(0, -1))));
        assert!(!game.is_valid_move(&Surprise(c(1, -1), c(2, -2))));
        assert!(!game.is_valid_move(&Surprise(c(2, -1), c(3, -2))));
        assert!(!game.is_valid_move(&Surprise(c(2, -1), c(2, -2))));

        // Can't start from empty spot.
        assert!(!game.is_valid_move(&Surprise(c(3, 3), c(0, 1))));
    }
}
