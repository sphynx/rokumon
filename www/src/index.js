import React from 'react';
import ReactDOM from 'react-dom';

import { Game } from './game.js'
import { GameSetup } from './start.js'

import './index.css';

class GameLoader extends React.Component {
  constructor(props) {
    super(props);
    this.state = { wasm: {}, wasm_loaded: false, game: {}, playground: {} };
  }

  render() {
    return this.state.wasm_loaded
      ? <Game
        game_data={this.state.game}
        playground={this.state.playground}
        bot_moves_first={this.state.bot_moves_first}
      />
      : <span>Loading wasm...</span>;
  }

  componentDidMount() {
    this.loadWasm();
  }

  loadWasm = async () => {
    try {
      const wasm = await import('rokumon_wasm');

      // TODO: get those from UI
      const enable_fight = false;
      const hex_grid = true;
      const bot_moves_first = false;
      const duration = 1;

      const opts = wasm.Opts.new(enable_fight, hex_grid, bot_moves_first, duration);

      let playground = wasm.Playground.new(opts);

      const game = playground.get_game();
      this.setState({ wasm, wasm_loaded: true, game, bot_moves_first, playground });

    } catch (err) {
      console.error(`Unexpected error in loadWasm. Message: ${err.message}`);
    }
  };
}

ReactDOM.render(
  <GameSetup />,
  document.getElementById('root')
);

