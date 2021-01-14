import React, { Component } from 'react';
import './App.css';
import Greet from './Greet';

class App extends Component {

  render() {
    return (
      <div className="App">
        <h2>Example application</h2>
        <Greet />
      </div>
    );
  }
}

export default App;
