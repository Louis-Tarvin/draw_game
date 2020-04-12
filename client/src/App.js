import React from 'react';
import './fonts.css';
import './App.css';

import { useSelector } from 'react-redux';

import Landing from './landing/Landing';
import Room from './room/Room';

function App({ socketManager }) {
    const inRoom = !!useSelector(state => state.room);
    const disconnected = useSelector(state => state.socketState === 'disconnected');

    const connectionBar = disconnected? (<div className="connection-bar">There seems to be a connection issue...</div>): null;

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
