# Implement my own AI (based on alpha-beta pruning)

`rubot` crate seems to be a good fit for the job:
https://docs.rs/rubot/0.2.0/rubot/trait.Game.html

# Make dice in player's stock multisets instead of vectors

We should ignore the order there, since it's not important in the game
and it will lead to the fact that applying move and then unndoing it
should be identity operation. However, currently it's not always so,
since sometimes we put a die back in player's supply in a different
place. If we ignore the order -- this will be eliminated.

# Add visualization of the game state

Generate an SVG image or something.

# Add a mode to play random games

This we can explore the game tree in depth, perhaps it will be useful
for exposing certain move generator bugs.

Also, it can help answer interesting questions about expected length
of random game, who is expected to win it, etc.

We can also experiment with different distributions, but for start the
uniform distribution over all possible moves should be fine.

# Provide a REPL for actually playing the game

Perhaps we will need to have some better text UI for that. Not sure
how to display the card position in text... ASCII graphics seems to be
too much work.

`rustyline` looks like a good way to do REPL:
https://crates.io/crates/rustyline

# Experiment with performance

We already have basic profiling with FlameGraph and dtrace working on
Mac.

It looks like surprising amount of time is spent in formatting errors
with formatting macros like `ensure!`.

Also, most of the time (more than 50%) was spent in `validate_move`
which is the way we generate moves (just generate lots of them and
then validate if they can actually be made). We may improve on that
and try to generate good moves in the first place, it may lead to
small improvements (small because we will still need to check similar
conditions whether the move is valid or not).

# Add benchmarks

Which can help us with choosing data structures and quickly seeing
effects of various changes.

# Take a look at fuzzing tests

`cargo fuzz` seems to be a way to go.

See also how fuzzing is done in `rubot`:
https://github.com/lcnr/rubot/tree/master/fuzz

https://rust-fuzz.github.io/book/cargo-fuzz.html

# See how perft is done in `shakmaty`

https://github.com/niklasf/shakmaty/blob/master/src/perft.rs

Perhaps I can learn something from there or contribute a parallel
perft implementation to `shakmaty`

# Implement Automa - an AI from the game rules used for solo play

# Implement UI (either Web-based or based on something like ggez or just some UI library).

If Web, we can potentially play with WebAssembly here.

# Figure out how to convert hex coord to brick-shaped rectangles
  (cards). We need back and forth translation.

# Figure out why I don't have full resolution on Retina.
https://github.com/ggez/ggez/issues/587

Looks complicated, perhaps it won't work at all with ggez :)

# DONE Figure out how to generate a new board using hex coordinates:

https://www.redblobgames.com/grids/hexagons/
https://www.redblobgames.com/grids/hexagons/implementation.html

# DONE Implement Hash trait for Coord - just derive.

# DONE Implement conversion between user coordinates and internal ones

`R1C2` (i.e. row 1, card 2: rows counted starting from 1 from the top,
cards counted starting from 1 from the left) and `<0, 0, 0>` (i.e.
`Coord{ x: 0, y: 0, z: 0}` in the code).

# DONE Implement actual game rules

Write functions:
- to check if a move is valid
- to return neighbouring position for a given position
- to check victory conditions
- to generate all adjacent-triples for current board
- to apply moves to the game state

# DONE Write a parser for moves

Format can be something like this:

- place W1 at R2C2
- move B3 from R1C1 to R1C2
- fight at R2C3
- surprise from R1C2 to <3, -2, -1>
- submit

Surprise is tricky for the user because it requires our coordinates.

So there should be a way to show the internal coordinates of available
positions while repositioning a particular card.

I think we should use `nom` for that.

# DONE Implement move generator

I.e. to generate all possible moves from given position. This is
useful for tests and required for the AI.

# DONE Parametrize a game with particular instances of the rules, used cards and board layout

This would allow to quickly experiment with different set of rules.

Already done for cards and layouts. Still have to do for GameRules
(currently just two booleans).

Perhaps Builder pattern can be handy here.

# DONE Provide command line arguments to experiment with the game

May be similar to my Hamisado game:
https://github.com/sphynx/hamisado/blob/master/Hamisado.hs

Useful options:

- play mode (Perft, Play, others)
- opponents (AIHuman, HumanAI, HumanHuman, AIAI, Analyse)
- rule variations (--disable-fight-move, --disable-surprise-move)
- starting position and layout (--cards=JJJGGGG --layout=Brick7), shuffled by default
- depth used for perft or analysis
- maybe seed for reproducibility

We should use `structopt` crate for this.

# DONE Better structure

Move coords, decks and game board logic into separate files, since
current game.rs is too large.

# DONE Parallelise Perft

It should be almost trivial: on depth 1 split the generated moves into
`ncore` parts, run `perft` on each of those buckets and add numbers
together.

Will need to understand what's the simplest way to it in Rust, most
likely `rayon` crate.

