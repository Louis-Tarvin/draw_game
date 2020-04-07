import React from 'react';
import './Room.css';

import Chat from './Chat';
import Canvas from './Canvas';

import { useSelector } from 'react-redux';

export default function Room({ socketManager }) {
    const room = useSelector(state => state.room);

    const isLeader = useSelector(state => state.room.leaderID === state.socketID);

    // Leader ID is sent shortly after joining room for first time
    if (!room.leaderID) {
        return (<></>);
    }

    let title;
    if (isLeader) {
        title = (<h2>Draw {room.word}</h2>);
    } else {
        title = (<h2>Guess what {room.users[room.leaderID].username} is drawing</h2>);
    }

    return (
        <div className="room">
            <h2>In room {room.code}</h2>
            {title}
            <Canvas socketManager={socketManager} isLeader={isLeader} />
            <Chat socketManager={socketManager} disabled={isLeader} />
        </div>
    );
}
