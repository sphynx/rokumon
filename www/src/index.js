import React from 'react';
import ReactDOM from 'react-dom';
import './index.css';
import _ from 'lodash';

function Card(props) {
  const kindClass = props.kind.toLowerCase();
  const selectedClass = props.selected ? 'selected-card' : '';
  const { x, y } = coord_to_position(props.grid, props.coord);
  const styles = {
    left: x,
    top: y,
  };

  return (
    <div
      style={styles}
      className={`card ${kindClass} ${selectedClass}`}
      onClick={props.onClick}>

      <span className="card-ix">{props.ix}</span>

      {props.dice.map(d =>
        <DieOnCard color={d.color} value={d.value} />
      )}

    </div>
  )
}

function Die(props) {
  const die_char = "⚀⚁⚂⚃⚄⚅"[props.value - 1];
  const selected_clz = props.selected ? 'selected-die' : '';
  return (
    <span
      className={`die ${props.color.toLowerCase() + '-die'} ${selected_clz}`}
      onClick={props.onClick}>
      {die_char}
    </span>
  )
}

function DieOnCard(props) {
  const die_char = "⚀⚁⚂⚃⚄⚅"[props.value - 1];
  return (
    <span className={`die-on-card ${props.color.toLowerCase() + '-die-on-card'}`}>
      {die_char}
    </span>
  );
}

function DiceStock(props) {
  return (
    <div className="dice-stock">
      {props.dice.map((d, ix) =>
        <Die
          key={ix}
          color={d.color}
          value={d.value}
          selected={d === props.selectedDie}
          onClick={() => props.onClick(d)}
        />
      )}
    </div>
  )
}

class Board extends React.Component {
  render() {
    return (
      <div className="card-board">
        {this.props.cards.map((coord_card, ix) => {
          const [coord, card] = coord_card;
          return (<Card
            key={coord}
            ix={ix}
            coord={coord}
            grid={this.props.grid}
            kind={card.kind}
            dice={card.dice}
            selected={coord === this.props.selectedCard}
            onClick={() => this.props.onClick(coord)}
          />);
        }
        )}
      </div>
    );
  }
}

function History(props) {
  return (
    <div>
      History:
      <ol>
        {props.moves.map((move, ix) =>
          <li>Move {ix}: {ppMove(move)}</li>
        )}
      </ol>
    </div>
  );
}

function GameInfo(props) {
  const status = "The game is about to start";
  const sel_die = props.selectedDie;
  const selected_die_info = sel_die ? sel_die.color + sel_die.value : 'None';

  return (
    <div className="game-info">
      <div>Status: {status}</div>
      <div>To move: {props.whoMoves}</div>
      <div>Selected card: {props.selectedCardInfo}</div>
      <div>Selected die: {selected_die_info}</div>
    </div>
  );
}

class Game extends React.Component {
  constructor(props) {
    super(props);

    const game = props.game_data;
    const board = game.board;
    const player1 = game.player1;
    const player2 = game.player2;
    const player1_moves = game.player1_moves;
    const history = game.history;

    this.state = {
      board: {
        grid: board.grid,
        cards: board.cards
      },
      player1_moves,
      player1,
      player2,
      history,

      selected_card: null,
      selected_die: null,
    }
  }

  updateAllGame(game) {
    const board = game.board;
    const player1 = game.player1;
    const player2 = game.player2;
    const player1_moves = game.player1_moves;
    const history = game.history;

    this.setState({
      board: {
        grid: board.grid,
        cards: board.cards
      },
      player1_moves,
      player1,
      player2,
      history,

      selected_card: null,
      selected_die: null,
    });
  }

  getCardAtCoord(board, target_coord) {
    const coord_card = board.cards.find((coord_card) => _.isEqual(coord_card[0], target_coord));
    return coord_card ? coord_card[1] : undefined;
  }

  deleteDie(dice, target_die) {
    dice.splice(dice.findIndex(die => _.isEqual(die, target_die)), 1);
  }

  handleDieClick(die) {
    this.setState({ selected_die: die, selected_card: null });
  }

  handleCardClick(coord) {
    if (this.state.selected_card === coord) {
      // Deselect previously selected card.
      this.setState({ selected_card: null });
    } else if (this.state.selected_die !== null) {
      // Place selected die on this card.
      const move = { 'Place': [this.state.selected_die, coord] };
      this.applyMove(move);
      this.sendMoveToBot(move);
      this.setState({ selected_die: null });
    } else if (this.state.selected_card !== null) {
      // Move a die from previously selected card to this one.
      const card = this.getCardAtCoord(this.state.board, this.state.selected_card);
      if (card.dice.length > 0) {
        let die = _.last(card.dice);
        const move = { 'Move': [die, this.state.selected_card, coord] };
        this.applyMove(move);
        this.sendMoveToBot(move);
        this.setState({ selected_die: null, selected_card: null });
      }
    } else {
      // Select first card.
      this.setState({ selected_die: null, selected_card: coord });
    }
  }

  applyMove(move) {
    const history = this.state.history.concat([move]);
    for (var kind in move) {

      switch (kind) {
        case 'Place': {
          const what = move[kind][0];
          const where = move[kind][1];

          // Delete die from this player's supply.
          if (this.state.player1_moves) {
            let player1_copy = _.cloneDeep(this.state.player1);
            this.deleteDie(player1_copy.dice, what);
            this.setState({ player1: player1_copy });
          } else {
            let player2_copy = _.cloneDeep(this.state.player2);
            this.deleteDie(player2_copy.dice, what);
            this.setState({ player2: player2_copy });
          }

          // Put the die on the card.
          let board_copy = _.cloneDeep(this.state.board);
          let card = this.getCardAtCoord(board_copy, where);
          card.dice.push(what);

          this.setState({
            board: board_copy,
            player1_moves: !this.state.player1_moves,
            history,
          });

          break;
        };

        case 'Move': {
          const what = move[kind][0];
          const from = move[kind][1];
          const to = move[kind][2];

          let board_copy = _.cloneDeep(this.state.board);

          // Delete die from one card
          let from_card = this.getCardAtCoord(board_copy, from);
          this.deleteDie(from_card.dice, what);

          // Put the die on another card.
          let to_card = this.getCardAtCoord(board_copy, to);
          to_card.dice.push(what);

          this.setState({
            board: board_copy,
            player1_moves: !this.state.player1_moves,
            history
          });

          break;
        };

        default:
          break;

      }
    }
  }

  getMoveFromBot() {
    const move = this.props.playground.get_move();
    this.applyMove(move);
    this.props.playground.send_move(move);
    this.setState({ ai_says: move })
  }

  sendLastMove() {
    const move = _.last(this.state.history);
    this.props.playground.send_move(move);
    const game_data = this.props.playground.get_game();
    this.updateAllGame(game_data);
  }

  sendMoveToBot(move) {
    this.props.playground.send_move(move);
    const game_data = this.props.playground.get_game();
    this.updateAllGame(game_data);
  }

  render() {
    const who_moves = this.state.player1_moves ? this.state.player1.name : this.state.player2.name;
    const sel_card_coord = this.state.selected_card;
    const card = this.getCardAtCoord(this.state.board, sel_card_coord);
    const selected_card_info = card ? card.kind : 'None';

    return (
      <div className="game">
        <button onClick={() => this.getMoveFromBot()}>Get bot's move</button>
        <div>Last bot move: {JSON.stringify(this.state.ai_says)}</div>
        <DiceStock
          dice={this.state.player2.dice}
          selectedDie={this.state.selected_die}
          onClick={(die) => this.handleDieClick(die)}
        />
        <Board
          cards={this.state.board.cards}
          grid={this.state.board.grid}
          selectedCard={this.state.selected_card}
          onClick={(coord) => this.handleCardClick(coord)}
        />
        <DiceStock
          dice={this.state.player1.dice}
          selectedDie={this.state.selected_die}
          onClick={(die) => this.handleDieClick(die)}
        />
        <GameInfo
          whoMoves={who_moves}
          selectedCardInfo={selected_card_info}
          selectedDie={this.state.selected_die}
        />
        <History moves={this.state.history} />
      </div>
    );
  }
}

class App extends React.Component {
  constructor(props) {
    super(props);
    this.state = { wasm: {}, wasm_loaded: false, game: {}, playground: {} };
  }

  render() {
    return this.state.wasm_loaded
      ? <Game game_data={this.state.game} playground={this.state.playground} />
      : <span>Loading wasm...</span>;
  }

  componentDidMount() {
    this.loadWasm();
  }

  loadWasm = async () => {
    console.log("loadWasm");
    try {
      const wasm = await import('rokumon_wasm');

      // TODO: get those from UI
      const opts = wasm.Opts.new(false, true);
      let playground = wasm.Playground.new(opts);
      const game = playground.get_game();
      this.setState({ wasm, wasm_loaded: true, game, playground });

    } catch (err) {
      console.error(`Unexpected error in loadWasm. Message: ${err.message}`);
    }
  };
}


// ========================================

ReactDOM.render(
  <App />,
  document.getElementById('root')
);

// Utils.

function ppMove(move) {
  for (var attr in move) {
    switch (attr) {
      case 'Place': {
        const what = move[attr][0];
        const where = move[attr][1];
        return 'place ' + ppDie(what) + ' at card with coordinates ' + JSON.stringify(where);
      }

      case 'Move': {
        const what = move[attr][0];
        const from = move[attr][1];
        const to = move[attr][2];
        return 'move ' + ppDie(what) + ' from card ' + JSON.stringify(from) + ' to card ' + JSON.stringify(to);
      }

      default:
        return 'unknown move';
    }
  }
}

function ppDie(die) {
  return die.color + die.value;
}

function coord_to_position(grid, coord) {
  if (grid === "Hex") {
    const scale_x = 45;
    const scale_y = 70;
    const shift_x = 20;
    const shift_y = 100;

    const q = coord.x;
    const r = coord.y;
    const sqt = Math.sqrt(3);

    return {
      x: shift_x + Math.round(scale_x * (sqt * q + sqt / 2 * r)),
      y: shift_y + Math.round(scale_y * (3 / 2 * r))
    }
  } else {
    const scale_x = 80;
    const scale_y = 100;
    const shift_x = 15;
    const shift_y = 100;

    return {
      x: shift_x + scale_x * coord.x,
      y: shift_y + scale_y * coord.y
    }
  }
}

