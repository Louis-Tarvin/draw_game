import React from 'react';
import './fonts.css';
import './App.css';

import { useSelector } from 'react-redux';

import Landing from './landing/Landing';
import Room from './room/Room';

function App({ socketManager }) {
    const inRoom = !!useSelector(state => state.room);
    const socketState = useSelector(state => state.socketState);
    // const socketState = 'disconnected'

    var connectionBar;
    switch (socketState) {
        case 'disconnected':
            connectionBar = (<div className="connection-bar">There seems to be a connection issue... Attempting to reconnect in 3 seconds</div>)
            break;
        case 'reconnecting':
            connectionBar = (<div className="connection-bar">Attempting to reconnect...</div>)
            break;
        default:
            connectionBar = null;
    }

    const view = inRoom? (<Room socketManager={socketManager} />): null;

    return (
        <div className="App">
            {connectionBar}
            <Landing socketManager={socketManager} isHidden={inRoom} />
            {view}
        </div>
    );
}

export default App;
