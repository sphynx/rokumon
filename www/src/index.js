import React from 'react';
import ReactDOM from 'react-dom';

import './index.css';

import { Game } from './game.js'
import { GameSetup } from './start.js'

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
      : <span>Loading WASM...</span>;
  }

  componentDidMount() {
    this.loadWasm(this.props.game_options);
  }

  loadWasm = async (game_opts) => {
    try {
      const wasm = await import('rokumon_wasm');

      const enable_fight = game_opts.level > 2;
      const hex_grid = game_opts.level !== 1;
      const bot_moves_first = !game_opts.playerGoesFirst;
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

class App extends React.Component {
  constructor(props) {
    super(props);

    // state: 'new' -> 'options collected' -> 'game loader' -> 'wasm loaded' -> 'game shown' -> 'game started' 
    this.state = { app_state: 'new', game_options: {} };
  }

  handleOptionsSubmit = (options) => {
    this.setState({ app_state: 'options_collected', game_options: options });
  }

  render() {
    if (this.state.app_state === 'new') {
      return <GameSetup onSubmit={this.handleOptionsSubmit} />;
    } else if (this.state.app_state === 'options_collected') {
      return <GameLoader game_options={this.state.game_options} />;
    }
  }
}


ReactDOM.render(
  <App />,
  document.getElementById('root')
);

