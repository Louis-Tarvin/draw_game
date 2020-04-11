import React from 'react';
import './fonts.css';
import './App.css';

import { useSelector } from 'react-redux';

import Landing from './landing/Landing';
import Room from './room/Room';

function App({ socketManager }) {
    const inRoom = !!useSelector(state => state.room);
    const socketState = useSelector(state => state.socketState);

    const try_reconnect = e => {
        e.preventDefault();

        socketManager.reconnect();
    }

    var connectionBar;
    switch (socketState) {
        case 'disconnected':
            connectionBar = (<div className="connection-bar">There seems to be a connection issue... Attempting to reconnect in 5 seconds</div>)
            break;
        case 'reconnecting':
            connectionBar = (<div className="connection-bar">Attempting to reconnect...</div>)
            break;
        case 'failed':
            connectionBar = (
                <div className="connection-bar">
                    Failed to reconnect.
                    <form className="reconnect-form" onSubmit={try_reconnect}>
                        <input type="submit" value="Try Again" />
                    </form>
                </div>
            )
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
