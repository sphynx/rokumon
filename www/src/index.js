import './index.css';

import React from 'react';
import ReactDOM from 'react-dom';
import { BrowserRouter, Switch, Route } from 'react-router-dom';

import { Game } from './game.js'
import { GameSetup } from './setup.js'

class GameLoader extends React.Component {
  constructor(props) {
    super(props);

    let game_opts;
    if (this.props.match !== undefined) {
      let { opponent, level, player_goes } = this.props.match.params;
      game_opts = { opponent, level: parseInt(level, 10), player_goes };
    } else {
      game_opts = { opponent: 'bot', level: 2, player_goes: 'first' };
    }

    const enable_fight = game_opts.level > 2;
    const hex_grid = game_opts.level !== 1;
    const bot_moves_first = game_opts.player_goes === "second";
    const duration = 1;

    let wasm = props.wasm;

    const opts = wasm.Opts.new(enable_fight, hex_grid, bot_moves_first, duration);
    let playground = wasm.Playground.new(opts);
    const game = playground.get_game();

    this.state = { game, bot_moves_first, playground };
  }

  render() {
    return (
      <Game
        game_data={this.state.game}
        playground={this.state.playground}
        bot_moves_first={this.state.bot_moves_first}
      />
    );
  }
}

// TODO: load wasm only once.
class WasmLoader extends React.Component {
  constructor(props) {
    super(props);
    this.state = { wasm: {}, wasm_loaded: false };
  }

  render() {
    return this.state.wasm_loaded
      ? <GameLoader wasm={this.state.wasm} match={this.props.match} />
      : <span>Loading WASM...</span>;
  }

  componentDidMount() {
    console.log("Loading WASM...");
    this.loadWasm();
  }

  loadWasm = async () => {
    try {
      const wasm = await import('rokumon_wasm');
      this.setState({ wasm, wasm_loaded: true });
    } catch (err) {
      console.error(`Unexpected error in loadWasm. Message: ${err.message}`);
    }
  }
}

const App = () => {
  return (
    <BrowserRouter basename="/rokumon">
      <Switch>
        <Route path="/game/:opponent/:level/:player_goes" component={WasmLoader} />
        <Route path="/default">
          <WasmLoader />
        </Route>
        <Route path="/">
          <GameSetup />
        </Route>
      </Switch>
    </BrowserRouter>
  );
}

ReactDOM.render(
  <App />,
  document.getElementById('root')
);
