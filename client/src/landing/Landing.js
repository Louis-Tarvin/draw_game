import React from 'react';
import classNames from 'classnames';
import useInput from 'common/useInput';

import './Landing.css';

function EnterRoom({ username, socketManager, enabled }) {
    const [roomCode, roomCodeField] = useInput({ placeholder: 'Room code' });

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
                <input type="submit" value="Join Room" />
            </form>
            <hr />
            <form className="create-room" onSubmit={createRoomSubmit}>
                <input type="submit" value="Create Room" />
            </form>
        </div>
    );
}

export default function Landing({ socketManager }) {
    const [username, usernameField] = useInput({ placeholder: 'Username' });
    const usernameIsValid = checkUsername(username);

    return (
        <div className="landing-wrapper">
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
