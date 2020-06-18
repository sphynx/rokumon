import React from 'react';

import './start.css';

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

  handleSubmit(event) {
  }

  render() {
    return (
      <div>
        <label>Pick your opponent:</label>
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
              <label>Watch</label>
              <label>Watch</label>
            </div>
          </div>
        </div>
      </div>
    );
  }
}
