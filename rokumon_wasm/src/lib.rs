mod utils;

use wasm_bindgen::prelude::*;

use rokumon::ai::AlphaBetaAI;
use rokumon::board::Layout;
use rokumon::card::Deck;
use rokumon::coord::Coord;
use rokumon::game::{Game, GameMove, Rules};
use rokumon::play::Strategy;

#[wasm_bindgen]
pub struct Opts {
    enable_fight: bool,
    hex_grid: bool,
    bot_goes_first: bool,
    duration: u8,
}

#[wasm_bindgen]
impl Opts {
    pub fn new(enable_fight: bool, hex_grid: bool, bot_goes_first: bool, duration: u8) -> Self {
        Self {
            enable_fight,
            hex_grid,
            bot_goes_first,
            duration,
        }
    }
}

#[wasm_bindgen]
pub struct Playground {
    ai: AlphaBetaAI,
    game: Game,
}

#[wasm_bindgen]
impl Playground {
    pub fn new(opts: Opts) -> Self {
        utils::set_panic_hook();

        let game = if opts.hex_grid {
            let deck = Deck::seven_shuffled();
            Game::new(Layout::Bricks7, deck, Rules::new(opts.enable_fight, false))
        } else {
            let deck = Deck::six_shuffled();
            Game::new(Layout::Rectangle6, deck, Rules::new(opts.enable_fight, false))
        };

        let ai = AlphaBetaAI::with_duration(opts.bot_goes_first, opts.duration as u64);
        Self { ai, game }
    }

    pub fn get_game(&self) -> JsValue {
        JsValue::from_serde(&self.game).unwrap()
    }

    pub fn get_move(&mut self) -> JsValue {
        let mov = self.ai.get_move(&self.game);
        self.game.apply_move(&mov).unwrap();
        JsValue::from_serde(&mov).unwrap()
    }

    pub fn validate_move(&self, mov_value: &JsValue) -> bool {
        let mov: GameMove<Coord> = mov_value.into_serde().unwrap();
        self.game.validate_move(&mov).is_ok()
    }

    pub fn send_move(&mut self, mov_value: &JsValue) {
        // FIXME: learn how to properly handle errors in WASM
        let mov: GameMove<Coord> = mov_value.into_serde().unwrap();
        self.game.apply_move(&mov).unwrap();
    }
}
