mod utils;

use wasm_bindgen::prelude::*;

use rokumon::board::Layout;
use rokumon::card::Deck;
use rokumon::game::{Game, Rules};

#[wasm_bindgen]
pub fn init_game(mode: u32) -> String {
    utils::set_panic_hook();
    if mode == 0 {
        let deck = Deck::seven_shuffled();
        let game = Game::new(Layout::Bricks7, deck, Rules::new(false, false));
        game.to_string()
    } else {
        let deck = Deck::ordered("jjjgggg").unwrap();
        let game = Game::new(Layout::Bricks7, deck, Rules::new(false, false));
        game.to_string()
    }
}
