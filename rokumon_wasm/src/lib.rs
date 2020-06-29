mod utils;

use wasm_bindgen::prelude::*;

use rokumon_core::ai::AlphaBetaAI;
use rokumon_core::board::Layout;
use rokumon_core::card::Deck;
use rokumon_core::coord::Coord;
use rokumon_core::game::{Game, GameMove, Rules};
use rokumon_core::play::Strategy;

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

// TODO: handle errors better by changing return types to Result<T, JSValue> and automating conversion
// of errors to JSValue (instead of just calling JsValue::from_str(&e.to_string())) all the time.
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
        self.game.apply_move(&mov).expect("get_move: Can't apply AI's move");
        JsValue::from_serde(&mov).expect("get_move: Serde serialization failed")
    }

    pub fn validate_move(&self, mov_value: &JsValue) -> Result<(), JsValue> {
        let mov: GameMove<Coord> = mov_value.into_serde().map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.game
            .validate_move(&mov)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub fn send_move(&mut self, mov_value: &JsValue) {
        let mov: GameMove<Coord> = mov_value.into_serde().unwrap();
        self.game.apply_move(&mov).unwrap();
    }
}
