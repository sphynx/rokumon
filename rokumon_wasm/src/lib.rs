mod utils;

use wasm_bindgen::prelude::*;

use rokumon::board::Layout;
use rokumon::card::Deck;
use rokumon::game::{Game, Rules};

#[wasm_bindgen]
pub struct Opts {
    enable_fight: bool,
    hex_grid: bool,
}

#[wasm_bindgen]

impl Opts {
    pub fn new(enable_fight: bool, hex_grid: bool) -> Self {
        Self { enable_fight, hex_grid }
    }
}

#[wasm_bindgen]
pub fn init_game(opts: Opts) -> JsValue {
    utils::set_panic_hook();
    let game = if opts.hex_grid {
        let deck = Deck::seven_shuffled();
        Game::new(Layout::Bricks7, deck, Rules::new(opts.enable_fight, false))
    } else {
        let deck = Deck::six_shuffled();
        Game::new(Layout::Rectangle6, deck, Rules::new(opts.enable_fight, false))
    };
    JsValue::from_serde(&game).unwrap()
}
