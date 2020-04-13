# Implement move generator

I.e. to generate all possible moves from given position. This is
useful for tests and required for the AI.

# Provide a REPL for actually playing the game - first just between two human players.

Perhaps we will need to have some better text UI for that. Not sure
how to display the card position in text... ASCII graphics seems to be
too much work. As the first approximation

`rustyline` looks like a good way to do REPL:
https://crates.io/crates/rustyline

# Parametrize a game with particular instances of the rules, used cards and board layout

This would allow to quickly experiment with different set of rules.

# Implement Automa - an AI from the game rules used for solo play

# Implement my own AI (based on alpha-beta pruning)

`rubot` crate seems to be a good fit for the job:
https://docs.rs/rubot/0.2.0/rubot/trait.Game.html

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
