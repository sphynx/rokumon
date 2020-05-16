# Rokumon

An AI for playing [Rokumon game](https://boardgamegeek.com/thread/2380440/wip-rokumon-2020-9-card-design-contest-contest-rea) by Charles Ward.

Rules of the game can also be found in this repo in [docs/rules.pdf](docs/rules.pdf)

# How to build, install and play

It's written in Rust programming language, so you have to run usual invocations for setting up Rust:

1. Install `rustup`, a standard installer for Rust (it supports variety of platforms including Linux, Mac and Windows):
https://rustup.rs/

2. Run `cargo build --release`. Cargo is package manager and build system for Rust which will download and build all depedencies and `rokumon` itself.

3. Then after some time you'll have a built `rokumon` executable in `target/release/rokumon`, you can run it directly:

```
➜  target/release/rokumon --help
rokumon 0.1.0

USAGE:
    rokumon [FLAGS] [OPTIONS]

FLAGS:
        --ai-to-completion           Whether AI should run its analysis to full completion before making the move
    -f, --enable-fight-move          Allows 'Fight' move in the game rules (disabled by default)
    -s, --enable-surprise-move       Allows 'Surprise' move in the game rules (disabled by default)
    -h, --help                       Prints help information
        --no-shuffle                 If we don't want to shuffle the deck before starting the game
        --second-ai-to-completion    Whether second AI should run its analysis to full completion before making the move
    -V, --version                    Prints version information

OPTIONS:
        --ai-depth <ai-depth>                        How deep AI should analyse the position (in plies, i.e. half-moves)
        --ai-duration <ai-duration>                  How much time AI should use for its moves (in seconds)
        --cards <cards>
            Cards to be used in the game (g - gold, j - jade) [default: gggjjjj]

    -l, --layout <layout>                            Layout: Bricks7 or Square6 (for Act I) [default: bricks7]
    -m, --mode <mode>                                play | perft | par_perft [default: play]
    -o, --opponents <opponents>
            HumanHuman | HumanAI | AIHuman | AIAI | RandomRandom [default: HumanAI]

        --perft-depth <perft-depth>                  Depth for performance tests [default: 5]
        --samples <samples>                          Number of matches to play (for AI vs AI games) [default: 10]
        --second-ai-depth <second-ai-depth>
            How deep second AI should analyse the position (in plies, i.e. half-moves)

        --second-ai-duration <second-ai-duration>    How much time second AI should use for its moves (in seconds)
```

4. Then you can play a game against an AI like this:

```
target/release/rokumon --opponents HumanAI
```

Note that you are much advised to build release version of the game (as opposed to `debug`), since AI in debug mode will be very weak.

There are multiple command line options described in `--help` which control AI, opponents and game rules.

You can, for example, ask two AIs to play against each other, control amount of time AI uses for its move, enable or disable 'fight' and 'surprise' moves in Rokumon rules and do other things.
