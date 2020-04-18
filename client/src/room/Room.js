import React from 'react';
import './Room.css';

import Chat from './Chat';
import Canvas from './Canvas';
import Lobby from './Lobby';

import { useSelector } from 'react-redux';

export default function Room({ socketManager }) {
    const word = useSelector(state => state.room.word);
    const roomCode = useSelector(state => state.room.code);
    const leader = useSelector(state => state.room.leader);
    const roomState = useSelector(state => state.room.state);
    const host = useSelector(state => state.room.host);
    const winner = useSelector(state => state.room.winner);

    const leaveRoomSubmit = e => {
        e.preventDefault();

        console.debug('Leaving room', roomCode);
        socketManager.leaveRoom();
    };

    // roomState is sent shortly after joining room for first time
    if (!roomState) {
        return (<></>);
    }

    let showCanvas = true;
    let showLobby = false;

    let title;
    switch (roomState) {
        case 'lobby':
            if (host.isCurrentUser) {
                title = (<h2 className="title">Press start when everyone is ready</h2>);
                showLobby = true;
            } else {
                title = (<h2 className="title">Waiting for {host.username} to start the game</h2>);
            }

            showCanvas = false;
            break;
        case 'leader':
            title = (<h2 className="title">Draw {word}</h2>);
            break;
        case 'guesser':
            title = (<h2 className="title">Guess what {leader.username} is drawing</h2>);
            break;
        case 'winner':
            if (winner.isCurrentUser) {
                title = (<h2 className="title">You guessed it!</h2>);
            } else {
                title = (<h2 className="title">{winner.username} correctly guessed the word</h2>);
            }
            break;
        case 'timeout':
            title = (<h2 className="title">Time has run out! The word was: {word}</h2>)
            break;
        default:
    }

    return (
        <div className="room">
            <div className="room-wrapper">
                <div className="canvas-card">
                    <div className="room-status-bar">
                        <h2>In room {roomCode}</h2>
                        <form className="leave-form" onSubmit={leaveRoomSubmit}>
                            <input type="submit" value="Leave Room" />
                        </form>
                    </div>
                    {title}
                    <div hidden={!showCanvas}>
                        <Canvas socketManager={socketManager} isLeader={roomState === 'leader'} />
                    </div>
                    <div hidden={!showLobby}>
                        <Lobby socketManager={socketManager} />
                    </div>
                </div>
                <div className="chat-card">
                    <Chat socketManager={socketManager} disabled={roomState === 'leader'} />
                </div>
            </div>
        </div>
    );
}
