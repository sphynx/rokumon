import './setup.css';

import React from 'react';
import { Link } from 'react-router-dom';

import bot_svg from './img/bot.svg';
import human_svg from './img/human.svg';
import watch_svg from './img/watch.svg';

export class GameSetup extends React.Component {
  constructor(props) {
    super(props);

    this.state = {
      opponents: 'bot',
      level: 3,
      player_goes_first: true,
    };
  }

  handleOpponentChange = (opponents) => {
    this.setState({ opponents });
  }

  handleTurnChange = (player_goes_first) => {
    this.setState({ player_goes_first });
  }

  handleLevelChange = (e) => {
    this.setState({ level: parseInt(e.target.value, 10) });
  }

  prepareUrl() {
    const turn = this.state.player_goes_first ? "first" : "second";
    return `/game/${this.state.opponents}/${this.state.level}/${turn}`;
  }

  render() {
    return (
      <div className="root">
        <div className="root-child">
          <Opponent onClick={this.handleOpponentChange} />
          <Level onInput={this.handleLevelChange} />
          <Turn onClick={this.handleTurnChange} playerGoesFirst={this.state.player_goes_first} />
          <Submit url={this.prepareUrl()} />
        </div>
      </div >
    );
  }
}

function Opponent(props) {
  return (
    <div className="option-block">
      <label className="option-label">Pick your opponent:</label>
      <div className="opponent">
        <div id="bot_opponent" className="opponent-choice" onClick={() => props.onClick("bot")}>
          <div>
            <img src={bot_svg} alt="bot" />
            <label>Bot</label>
          </div>
        </div>
        <div id="human_opponent" className="opponent-choice" onClick={() => props.onClick("human")}>
          <div>
            <img src={human_svg} alt="human" />
            <label>Human</label>
          </div>
        </div>
        <div id="watch" className="opponent-choice" onClick={() => props.onClick("watch")}>
          <div>
            <img src={watch_svg} alt="watch bots" />
            <label>Watch bots</label>
          </div>
        </div>
      </div>
    </div>
  );
}

function Level(props) {
  return (
    <div className="option-block">
      <label className="option-label">Select game level:</label>
      <select defaultValue="3" onInput={(e) => props.onInput(e)}>
        <option value="1">Level 1 - Six cards, no Fight, no Surprise</option>
        <option value="2">Level 2 - Seven cards, no Fight, no Surprise</option>
        <option value="3">Level 3 - Seven cards, no Surprise</option>
        <option value="4">Level 4 - Seven cards, all moves</option>
        <option value="5">Level 5 - Seven cards with Fort, all moves</option>
        <option value="6">Level 6 - Automa</option>
      </select>
    </div>
  );
}

function Turn(props) {
  const firstClass = props.playerGoesFirst ? "selected" : "";
  const secondClass = props.playerGoesFirst ? "" : "selected";

  return (
    <div className="option-block">
      <label className="option-label">Choose move order. You go:</label>
      <div className="turn">
        <div id="first" className={`turn-button ${firstClass}`} onClick={() => props.onClick(true)}>
          <span>First</span>
          <span>You'll have 4 dice</span>
        </div>
        <div id="second" className={`turn-button ${secondClass}`} onClick={() => props.onClick(false)}>
          <span>Second</span>
          <span>You'll have 5 dice</span>
        </div>
      </div>
    </div>
  );
}

function Submit(props) {
  return (
    <div className="submit-block">
      <div className="submit-btn">
        <Link to={props.url} className="submit-link">Go!</Link>
      </div>
    </div>
  );
}
