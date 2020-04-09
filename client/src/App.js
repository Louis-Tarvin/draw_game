import React from 'react';
import './fonts.css';
import './App.css';

import { useSelector } from 'react-redux';

import Landing from './landing/Landing';
import Room from './room/Room';

function App({ socketManager }) {
    const inRoom = !!useSelector(state => state.room);

    const view = inRoom? (<Room socketManager={socketManager} />): null;

    return (
        <div className="App">
            <Landing socketManager={socketManager} isHidden={inRoom} />
            {view}
        </div>
    );
}

export default App;
