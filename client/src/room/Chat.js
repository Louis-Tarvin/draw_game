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
        case 'timeout':
            return (
                <div className="message timeout">
                    Everyone ran out of time! The word was {message.word}
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
    const roomState = useSelector(state => state.room.state);
    const users = useSelector(state => state.room.users);
    const messages = useSelector(state => state.room.messages)
        .map((message, index) => (<Message key={index} message={message} users={users}/>));
    const autoscrollButtonRef = useRef(null);
    const messageRef = useRef(null);
    const chatBoxRef = useRef(null);

    // Since this effect has a dependency on messages this will run whenever a new message comes in
    useEffect(() => {
        const messagesElement = messageRef.current;
        if (messagesElement) {
            if (autoscroll) {
                // Simply scroll to the bottom, my avoiding using scrollIntoView
                // this prevents the whole browser window moving down as this only
                // scrolls within the messages element and nothing else.
                messagesElement.scrollTop = messagesElement.scrollHeight - messagesElement.clientHeight
            }
        }
        if (!disabled) {
            chatBoxRef.current.focus();
        }
    }, [messages, messageRef, autoscroll, disabled, chatBoxRef]);

    const chatSubmit = useCallback(e => {
        e.preventDefault();

        if (message !== '') {
            socketManager.sendChat(message);
        }

        setMessage('');
    }, [message, socketManager]);

    const enableAutoscroll = useCallback(() => {
        setAutoscroll(true);
    }, [setAutoscroll]);

    const disableAutoscroll = useCallback(() => {
        // Checking if last message is visible
        const lastMessage = messageRef.current.lastChild;
        const rect = lastMessage.getBoundingClientRect();
        const isNotVisible = (rect.top - messageRef.current.getBoundingClientRect().bottom >= 0);
        if (isNotVisible) {
            setAutoscroll(false);
        }
    }, [setAutoscroll, messageRef]);

    const autoscrollButtonClass = autoscroll ? "autoscroll-button invisible" : "autoscroll-button"

    let chatPlaceholder;
    switch (roomState) {
        case 'lobby':
            chatPlaceholder = "Chat to others in the room";
            break;
        case 'leader':
            chatPlaceholder = "Can't chat while drawing";
            break;
        case 'guesser':
            chatPlaceholder = "Make a guess";
            break;
        default:

    }

    return (
        <div className="chat-area">
            <input type="button"
                value="Resume Auto-scroll"
                className={autoscrollButtonClass}
                onClick={enableAutoscroll}
                ref={autoscrollButtonRef} />
            <div className="messages" onWheel={disableAutoscroll} ref={messageRef}>{messages}</div>
            <form className="chat-form" onSubmit={chatSubmit}>
                <input type="text"
                    ref={chatBoxRef}
                    value={disabled ? '' : message}
                    placeholder={chatPlaceholder}
                    onChange={e => setMessage(e.target.value)}
                    disabled={disabled}/>
            </form>
        </div>
    );
}
