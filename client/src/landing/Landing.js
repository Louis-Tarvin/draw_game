import React, { useEffect, useRef } from 'react';
import { useSelector } from 'react-redux';
import classNames from 'classnames';
import useInput from 'common/useInput';

import './Landing.css';

function EnterRoom({ username, socketManager, enabled }) {
    const [roomCode, roomCodeField] = useInput({ placeholder: 'Room code', maxlength: 5 });
    const disabled = useSelector(state => state.socketState !== 'connected');

    const joinRoomSubmit = e => {
        e.preventDefault();

        console.debug('Joining room', roomCode, 'with username', username);
        socketManager.joinRoom(roomCode, username);
    };

    const createRoomSubmit = e => {
        e.preventDefault();
        console.debug('Creating room with username', username);
        socketManager.createRoom(username);
    };

    return (
        <div className={classNames('enter-room', { 'show': enabled })}>
            <h2>Enter a room:</h2>
            <form className="join-room" onSubmit={joinRoomSubmit}>
                {roomCodeField}
                <input type="submit" value="Join Room" disabled={disabled} />
            </form>
            <hr />
            <form className="create-room" onSubmit={createRoomSubmit}>
                <input type="submit" value="Create Room" disabled={disabled} />
            </form>
        </div>
    );
}

export default function Landing({ socketManager, isHidden }) {
    const usernameInputRef = useRef(null);
    const [username, usernameField] = useInput({ placeholder: 'Username', ref: usernameInputRef, maxlength: 14 });
    const usernameIsValid = checkUsername(username);

    useEffect(() => {
        usernameInputRef.current.focus();
    }, [usernameInputRef]);

    const wrapperClass = isHidden? "hide landing-wrapper": "landing-wrapper";

    return (
        <div className={wrapperClass}>
            <div className="landing">
                <h2>Enter a username</h2>
                {usernameField}
                <p className={classNames('message', {
                    error: !usernameIsValid && username.length !== 0,
                    success: usernameIsValid,
                })}>
                    Your username must only contain letters,
                    numbers and the ‘.’ or ‘_’ symbols and it must be less
                    than 15 characters.
                </p>

                <EnterRoom
                    username={username}
                    socketManager={socketManager}
                    enabled={usernameIsValid} />
            </div>
        </div>
    );
}

function checkUsername(username) {
    return /^([a-zA-Z0-9]|\.|_)+$/.test(username) && username.length < 15;
}
