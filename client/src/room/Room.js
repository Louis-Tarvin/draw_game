import React from 'react';
import './Room.css';

import Chat from './Chat';
import Canvas from './Canvas';

import { useSelector } from 'react-redux';

export default function Room({ socketManager }) {
    const word = useSelector(state => state.room.word);
    const roomCode = useSelector(state => state.room.code);
    const leader = useSelector(state => state.room.leader);
    const isLeader = useSelector(state => state.room.isLeader);

    // Leader is sent shortly after joining room for first time
    if (!leader) {
        return (<></>);
    }

    let title;
    if (isLeader) {
        title = (<h2 className="title">Draw {word}</h2>);
    } else {
        title = (<h2 className="title">Guess what {leader.username} is drawing</h2>);
    }

    return (
        <div className="room">
            <div className="room-wrapper">
                <div className="canvas-card">
                    <h2>In room {roomCode}</h2>
                    {title}
                    <Canvas socketManager={socketManager} isLeader={isLeader} />
                </div>
                <div className="chat-card">
                    <Chat socketManager={socketManager} disabled={isLeader} />
                </div>
            </div>
        </div>
    );
}