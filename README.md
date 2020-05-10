# Rokumon

An AI for playing [Rokumon game](https://boardgamegeek.com/thread/2380440/wip-rokumon-2020-9-card-design-contest-contest-rea) by Charles Ward.

# How to build, install and play

It's written in Rust programming language, so you have to run usual invocations for settin up Rust:

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
    -f, --enable-fight-move
    -s, --enable-surprise-move
    -h, --help                    Prints help information
        --shuffle
    -V, --version                 Prints version information

OPTIONS:
        --ai-depth <ai-depth>
        --ai-duration <ai-duration>
        --cards <cards>                               [default: gggjjjj]
    -d, --depth <depth>                               [default: 5]
    -l, --layout <layout>                             [default: bricks7]
    -m, --mode <mode>                                 perft | par_perft | play [default: play]
    -o, --opponents <opponents>                       HumanHuman | HumanAI | AIHuman | AIAI | RR [default: RR]
        --samples <samples>                           [default: 10]
        --second-ai-depth <second-ai-depth>
        --second-ai-duration <second-ai-duration>
```

4. Then you can play a game against AI like this:

```
target/release/rokumon --ai-duration 20 --opponents HumanAI --shuffle
```

There are multiple command line options described in `--help` which control AI, opponents and game rules.

You can, for example, ask two AIs to play against each other, control amount of time AI uses for its move, enable or disable 'fight' and 'surprise' moves in Rokumon rules and do other things.