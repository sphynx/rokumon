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
        <fieldset id="opponent">
          <div id="bot_opponent" className="opponent_choice">
            <div>
              <img src="/screens/start-screen/bot.svg" />
              <div>Bot</div>
            </div>
          </div>
          <div id="human_opponent" className="opponent_choice">
            <img src="/screens/start-screen/human.svg" />
            Human
          </div>
          <div id="watch" className="opponent_choice">
            <img src="/screens/start-screen/watch.svg" />

          </div>
        </fieldset>
      </div>
    );
  }
}
