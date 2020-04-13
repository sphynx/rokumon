mod game;
mod parsers;

fn main() {
    println!("Rokumon, v0.1");

    let game = game::Game::new();
    println!("{}", game);
}
