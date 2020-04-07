import React, { useCallback, useState } from 'react';
import { useSelector } from 'react-redux';

function Message({ message, users }) {
    switch (message.type) {
        case 'initial_join':
        return (
            <div className="message initial">
                Welcome to room
                <span className="room-code"> {message.roomCode}</span>,
                current connected users:
                <span className="username-list">
                    {' ' + Object.values(message.users).map(user => user.username).join(', ')}
                </span>
            </div>
        );
        case 'chat':
            return (
                <div className="message">
                    <span className="username">{users[message.userID].username}: </span>
                    <span className="content">{message.content}</span>
                </div>
            );
        case 'winner':
            return (
                <div className="message winner">
                    <span className="username">{users[message.winnerID].username} </span>
                    correctly guessed the word
                    <span className="word"> {message.word}</span>
                </div>
            );
        case 'user_join':
            return (
                <div className="message user-joined">
                    <span className="username">{users[message.userID].username} </span>
                    joined the room
                </div>
            );
        case 'user_left':
            return (
                <div className="message user-left">
                    <span className="username">{message.username}</span>
                    left the room
                </div>
            );
        default:
            console.warn('Unhandled message type');
            return (<div className="message error">Unhandled message type</div>);
    }
}

export default function Chat({ socketManager, disabled }) {
    const [message, setMessage] = useState('');
    const users = useSelector(state => state.room.users);
    const messages = useSelector(state => state.room.messages)
        .map((message, index) => (<Message key={index} message={message} users={users}/>));

    const chatSubmit = useCallback(e => {
        e.preventDefault();

        socketManager.sendChat(message);
        setMessage('');
    }, [message, socketManager]);

    return (
        <div className="chat-area">
            <div className="messages">{messages}</div>
            <form className="chat-form" onSubmit={chatSubmit}>
                <input type="text"
                    value={message}
                    placeholder={disabled ? "You aren't allowed to guess and draw": "Make a guess"}
                    onChange={e => setMessage(e.target.value)}
                    disabled={disabled}/>
            </form>
        </div>
    );
}
