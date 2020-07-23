import './game.css';

import React from 'react';
import _ from 'lodash';

import red0 from './img/red0.svg'
import red2 from './img/red2.svg'
import red4 from './img/red4.svg'
import red6 from './img/red6.svg'

import black0 from './img/black0.svg'
import black1 from './img/black1.svg'
import black3 from './img/black3.svg'
import black5 from './img/black5.svg'
import black_star from './img/black-star.svg'

function Card(props) {
  const kindClass = props.kind.toLowerCase();
  const selectedClass = props.selected ? 'selected' : '';
  const { x, y } = coordToPosition(props.grid, props.coord);
  const styles = { left: x, top: y };
  return (
    <div
      style={styles}
      className={`card ${kindClass} ${selectedClass}`}
      onClick={props.onClick}>

      {props.dice.map((d, ix) =>
        <DieOnCard key={ix} color={d.color} value={d.value} />
      )}
    </div>
  );
}

function Die(props) {
  const selected_clz = props.selected ? 'selected' : '';
  return (
    <img
      src={dieImage(props.color, props.value)}
      onClick={props.onClick}
      alt={`${props.color} die with value ${props.value}`}
      className={`die ${selected_clz}`}
    />
  );
}

function DieOnCard(props) {
  const selected_clz = props.selected ? 'selected' : '';
  return (
    <img
      src={dieImage(props.color, props.value)}
      onClick={props.onClick}
      alt={`${props.color} die with value ${props.value}`}
      className={`die ${selected_clz}`}
    />
  );
}

function DiceStock(props) {
  return (
    <div className={`dice-stock ${props.className}`}>
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
  );
}

function Board(props) {
  const styles = {
    // Hex:       4 x card_width + 3 * horiz_gap
    // Rectangle: 3 x card_width + 2 * horiz_gap

    // We have to specify this manually in JS, since there is
    // no easy way to calculate the size taken by absolutely positioned elements
    // (cards).
    "minWidth": props.grid === "Hex" ? "315px" : "235px",
  };
  return (
    <div className="card-board" style={styles}>
      {
        props.cards.map((coord_card, ix) => {
          const [coord, card] = coord_card;
          return (<Card
            key={ix}
            ix={ix}
            coord={coord}
            grid={props.grid}
            kind={card.kind}
            dice={card.dice}
            selected={coord === props.selectedCard}
            onClick={() => props.onClick(coord)}
          />);
        }
        )
      }
    </div>
  );
}

function History(props) {
  return (
    <div className="history">
      History:
      <ol>
        {props.moves.map((move, ix) =>
          <li key={ix}>Move {ix}: {ppMove(move)}</li>
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

function Fight(props) {
  return props.isOn ?
    (
      <button type="button" className="fight" onClick={props.onClick}>
        Fight
      </button>
    )
    : null;
}

const game_result = {
  IN_PROGRESS: "In progress",
  YOU_WON: "You won",
  BOT_WON: "Bot won",
}

export class Game extends React.Component {
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

  you() {
    return this.props.bot_moves_first ? this.state.player2 : this.state.player1;
  }

  bot() {
    return this.props.bot_moves_first ? this.state.player1 : this.state.player2;
  }

  canFight() {
    if (this.state.selected_card) {
      let card = this.getCardAtCoord(this.state.board, this.state.selected_card);
      if (card.dice.length === 2) {
        const first_owner = dieBelongsToPlayer1(card.dice[0]);
        const second_owner = dieBelongsToPlayer1(card.dice[1]);
        return (this.state.player1_moves === first_owner)
          || (this.state.player1_moves === second_owner);
      } else {
        return false;
      }
    } else {
      return false;
    }
  }

  botToMove() {
    return (this.state.player1_moves === this.props.bot_moves_first) && this.isGameInProgress();
  }

  getCardAtCoord(board, target_coord) {
    const coord_card = board.cards.find((coord_card) => _.isEqual(coord_card[0], target_coord));
    return coord_card ? coord_card[1] : undefined;
  }

  deleteDie(dice, target_die) {
    dice.splice(dice.findIndex(die => _.isEqual(die, target_die)), 1);
  }

  handleDieClick(die) {
    this.setState((state, props) => {
      // Can only select our own dice.
      if (state.player1_moves === dieBelongsToPlayer1(die)) {
        return { selected_die: state.selected_die === die ? null : die, selected_card: null };
      } else {
        return {};
      }
    });
  }

  handleCardClick(coord) {
    if (this.state.selected_card === coord) {
      // Deselect previously selected card.
      this.setState({ selected_card: null });
    } else if (this.state.selected_die !== null) {
      // Place selected die on this card.
      const move = { 'Place': [this.state.selected_die, coord] };
      this.handleMove(move);
    } else if (this.state.selected_card !== null) {
      // Move a die from previously selected card to this one.
      const card = this.getCardAtCoord(this.state.board, this.state.selected_card);
      if (card.dice.length > 0) {
        let die = _.last(card.dice);
        const move = { 'Move': [die, this.state.selected_card, coord] };
        this.handleMove(move);
      } else {
        this.setState({ selected_card: null });
      }
    } else {
      // Select first card.
      this.setState({ selected_die: null, selected_card: coord });
    }
  }

  handleFightClick() {
    if (this.state.selected_card) {
      const move = { 'Fight': this.state.selected_card };
      this.handleMove(move);
    }
  }

  handleMove(move) {
    try {
      this.validateMove(move);
      this.applyMove(move);
      this.sendMoveToBot(move);
      this.checkGameResult();
      this.setState({ selected_die: null, selected_card: null });
    } catch (e) {
      alert(e);
      this.setState({ selected_card: null, selected_die: null });
    }
  }

  applyMove(move) {
    const history = this.state.history.concat([move]);
    for (var kind in move) {
      switch (kind) {
        case 'Place': {
          const what = move[kind][0];
          const where = move[kind][1];
          this.setState((state) => {
            return this.applyPlaceMove(state, what, where, history)
          });
          break;
        };

        case 'Move': {
          const what = move[kind][0];
          const from = move[kind][1];
          const to = move[kind][2];
          this.setState((state) => {
            return this.applyMoveMove(state, what, from, to, history)
          });
          break;
        };

        case 'Fight': {
          const where = move[kind];
          this.setState((state) => {
            return this.applyFightMove(state, where, history)
          });
          break;
        }

        default:
          break;
      }
    }
  }

  applyPlaceMove(state, what, where, history) {
    let player1_copy = _.cloneDeep(state.player1);
    let player2_copy = _.cloneDeep(state.player2);

    // Delete die from this player's supply.
    if (state.player1_moves) {
      this.deleteDie(player1_copy.dice, what);
    } else {
      this.deleteDie(player2_copy.dice, what);
    }

    // Put the die on the card.
    let board_copy = _.cloneDeep(state.board);
    let card = this.getCardAtCoord(board_copy, where);
    card.dice.push(what);

    return {
      board: board_copy,
      player1: player1_copy,
      player2: player2_copy,
      player1_moves: !state.player1_moves,
      history,
    };
  }

  applyMoveMove(state, what, from, to, history) {
    let board_copy = _.cloneDeep(state.board);

    // Delete die from one card
    let from_card = this.getCardAtCoord(board_copy, from);
    this.deleteDie(from_card.dice, what);

    // Put the die on another card.
    let to_card = this.getCardAtCoord(board_copy, to);
    to_card.dice.push(what);

    return {
      board: board_copy,
      player1_moves: !state.player1_moves,
      history,
    };
  };

  applyFightMove(state, where, history) {
    let board_copy = _.cloneDeep(state.board);
    let player1_copy = _.cloneDeep(state.player1);
    let player2_copy = _.cloneDeep(state.player2);

    let card = this.getCardAtCoord(board_copy, where);
    const loser = firstIsLoser(card.dice[0], card.dice[1]) ? card.dice[0] : card.dice[1];

    this.deleteDie(card.dice, loser);

    if (dieBelongsToPlayer1(loser)) {
      player1_copy.dice.push(loser);
    } else {
      player2_copy.dice.push(loser);
    }

    return {
      board: board_copy,
      player1: player1_copy,
      player2: player2_copy,
      player1_moves: !state.player1_moves,
      history,
    };
  }

  getMoveFromBot() {
    const move = this.props.playground.get_move();
    this.applyMove(move);
    this.checkGameResult();
  }

  sendMoveToBot(move) {
    this.props.playground.send_move(move);
  }

  validateMove(move) {
    if (this.isGameInProgress()) {
      return this.props.playground.validate_move(move);
    } else {
      throw new Error(
        "The game is finished. Use 'Back' button to go back to the start screen."
      );
    }
  }

  getGameResult() {
    const game = this.props.playground.get_game();

    if (game.result === "FirstPlayerWon") {
      return this.props.bot_moves_first ? game_result.BOT_WON : game_result.YOU_WON;
    } else if (game.result === "SecondPlayerWon") {
      return this.props.bot_moves_first ? game_result.YOU_WON : game_result.BOT_WON;
    } else {
      return game_result.IN_PROGRESS;
    }
  }

  isGameInProgress() {
    return this.getGameResult() === game_result.IN_PROGRESS;
  }

  checkGameResult() {
    const status = this.getGameResult();
    if (status === game_result.YOU_WON) {
      alert('Congratulations, you won!');
    } else if (status === game_result.BOT_WON) {
      alert('The bot won this game.');
    }
  }

  render() {
    const who_moves = this.state.player1_moves ? this.state.player1.name : this.state.player2.name;
    const sel_card_coord = this.state.selected_card;
    const card = this.getCardAtCoord(this.state.board, sel_card_coord);
    const selected_card_info = card ? card.kind : 'None';

    return (
      <div className="game">
        <DiceStock
          dice={this.bot().dice}
          selectedDie={this.state.selected_die}
          onClick={(die) => this.handleDieClick(die)}
          className="top"
        />
        <Board
          cards={this.state.board.cards}
          grid={this.state.board.grid}
          selectedCard={this.state.selected_card}
          onClick={(coord) => this.handleCardClick(coord)}
        />
        <DiceStock
          dice={this.you().dice}
          selectedDie={this.state.selected_die}
          onClick={(die) => this.handleDieClick(die)}
          className="bottom"
        />
        <GameInfo
          whoMoves={who_moves}
          selectedCardInfo={selected_card_info}
          selectedDie={this.state.selected_die}
        />
        <Fight isOn={this.canFight()} onClick={() => this.handleFightClick()} />
        <History moves={this.state.history} />
      </div>
    );
  }

  componentDidMount() {
    if (this.botToMove()) {
      this.getMoveFromBot();
    }
  }

  componentDidUpdate(prevProps, prevState) {
    if (this.state.history.length > prevState.history.length) {
      if (this.botToMove()) {
        setTimeout(() => this.getMoveFromBot(), 20);
      }
    }
  }
}

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

      case 'Fight': {
        const where = move[attr];
        return 'fight at ' + JSON.stringify(where);
      }

      default:
        return 'unknown move';
    }
  }
}

function coordToPosition(grid, coord) {
  const card_width = 75;
  const card_height = 110;
  const horizontal_gap = 5; // px
  const vertical_gap = 5; // px

  const size_x = card_width + horizontal_gap;
  const size_y = card_height + vertical_gap;

  const shift_x = 0;
  const shift_y = size_y;

  if (grid === "Hex") {
    return {
      x: shift_x + Math.round(size_x * (coord.x + coord.y / 2)),
      y: shift_y + size_y * coord.y
    }
  } else {
    return {
      x: shift_x + size_x * coord.x,
      y: shift_y + size_y * coord.y
    }
  }
}

function dieBelongsToPlayer1(die) {
  return die.color === 'Red';
}

function dieImage(color, value) {
  if (color === 'Red' && value === 2) {
    return red2;
  } else if (color === 'Red' && value === 4) {
    return red4;
  } else if (color === 'Red' && value === 6) {
    return red6;
  } else if (color === 'Red' && value === 0) {
    return red0;
  } else if (color === 'Black' && value === 1) {
    return black1;
  } else if (color === 'Black' && value === 3) {
    return black3;
  } else if (color === 'Black' && value === 5) {
    return black5;
  } else if (color === 'Black' && value === 0) {
    return black0;
  } else if (color === 'White' && value === 1) {
    return black_star;
  };
}

function ppDie(die) {
  return die.color + die.value;
}

function firstIsLoser(die1, die2) {
  if (die1.color === 'White' && die2.value === 6) {
    return false;
  } else if (die2.color === 'White' && die1.value === 6) {
    return true;
  } else {
    return die1.value < die2.value;
  }
}
