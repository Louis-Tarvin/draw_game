import React, { useCallback, useState, useRef, useEffect } from 'react';
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
                    <span className="username">{message.user.username}: </span>
                    <span className="content">{message.content}</span>
                </div>
            );
        case 'winner':
            return (
                <div className="message winner">
                    <span className="username">{message.winner.username} </span>
                    correctly guessed the word
                    <span className="word"> {message.word}</span>
                </div>
            );
        case 'user_join':
            return (
                <div className="message user-joined">
                    <span className="username">{message.user.username} </span>
                    joined the room
                </div>
            );
        case 'user_left':
            return (
                <div className="message user-left">
                    <span className="username">{message.user.username} </span>
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
    const [autoscroll, setAutoscroll] = useState(true);
    const users = useSelector(state => state.room.users);
    const messages = useSelector(state => state.room.messages)
        .map((message, index) => (<Message key={index} message={message} users={users}/>));
    const autoscrollButtonRef = useRef(null);
    const messageRef = useRef(null);

    useEffect(() => {
        const messages = messageRef.current
        if (messages) {
            const lastMessage = messages.lastChild;
            if (autoscroll) {
                if (lastMessage.scrollIntoView) {
                    lastMessage.scrollIntoView({ behavior: 'smooth' });
                } else {
                    messages.scrollTop = messages.scrollHeight;
                }
            }
        }
    }, [messages, messageRef, autoscroll]);

    const chatSubmit = useCallback(e => {
        e.preventDefault();

        socketManager.sendChat(message);
        setMessage('');
    }, [message, socketManager]);

    const enableAutoscroll = useCallback(() => {
        setAutoscroll(true);
    }, [setAutoscroll]);

    const disableAutoscroll = useCallback(() => {
        console.log("autoscroll disabled");
        setAutoscroll(false);
    }, [setAutoscroll]);

    const autoscrollButton = autoscroll ? null : <input type="button"
        value="Resume Auto-scroll"
        className="autoscroll-button"
        onClick={enableAutoscroll}
        ref={autoscrollButtonRef} />

    return (
        <div className="chat-area">
            {autoscrollButton}
            <div className="messages" onWheel={disableAutoscroll} ref={messageRef}>{messages}</div>
            <form className="chat-form" onSubmit={chatSubmit}>
                <input type="text"
                    value={disabled ? '' : message}
                    placeholder={disabled ? "Can't chat while drawing": "Make a guess"}
                    onChange={e => setMessage(e.target.value)}
                    disabled={disabled}/>
            </form>
        </div>
    );
}
