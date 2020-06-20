import React from 'react';

import './start.css';

class Opponent extends React.Component {
  constructor(props) {
    super(props);
  }

  render() {
    return (
      <div className="option-block">
        <label className="option-label">Pick your opponent:</label>
        <div className="opponent">
          <div id="bot_opponent" className="opponent-choice">
            <div>
              <img src="/screens/start-screen/bot.svg" />
              <label>Bot</label>
            </div>
          </div>
          <div id="human_opponent" className="opponent-choice">
            <div>
              <img src="/screens/start-screen/human.svg" />
              <label>Human</label>
            </div>
          </div>
          <div id="watch" className="opponent-choice">
            <div>
              <img src="/screens/start-screen/watch.svg" />
              <label>Watch bots</label>
            </div>
          </div>
        </div>
      </div>
    );
  }
}

class Level extends React.Component {
  constructor(props) {
    super(props);
  }

  render() {
    return (
      <div className="option-block">
        <label className="option-label">Select game level:</label>
        <select>
          <option>Level 1 - Six cards, no Fight, no Surprise</option>
          <option>Level 2 - Seven cards, no Fight, no Surprise</option>
          <option>Level 3 - Seven cards, no Surprise</option>
          <option>Level 4 - Seven cards, all moves</option>
          <option>Level 5 - Seven cards with Fort, all moves</option>
          <option>Level 6 - Automa</option>
        </select>
      </div>
    );
  }
}

class Turn extends React.Component {
  constructor(props) {
    super(props);
  }

  render() {
    return (
      <div className="option-block">
        <label className="option-label">Choose move order. You go:</label>
        <div className="turn">
          <div class="turn-button">
            <span>First</span>
            <span>You'll have 4 dice</span>
          </div>
          <div class="turn-button">
            <span>Second</span>
            <span>You'll have 5 dice</span>
          </div>
        </div>
      </div>
    );
  }
}

class Submit extends React.Component {
  constructor(props) {
    super(props);
  }

  render() {
    return (
      <div className="submit-parent">
        <div className="submit-btn">
          <span>Go!</span>
        </div>
      </div>
    );
  }
}

export class GameSetup extends React.Component {
  constructor(props) {
    super(props);
    this.state = {
      opponents: "HumanAI",
      level: 3,
      playerGoesFirst: true,
    };

    this.handleInputChange = this.handleInputChange.bind(this);
  }

  handleInputChange(event) {
    this.setState({
      opponents: event.target.value
    });
  }

  render() {
    return (
      <div className="root">
        <div className="root-child">
          <Opponent />
          <Level />
          <Turn />
          <Submit />
        </div>
      </div >
    );
  }
}
