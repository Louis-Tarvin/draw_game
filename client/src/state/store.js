import { createStore } from 'redux';

function pushItem(array, item) {
    return [...array, item];
}

function stateManager(state = {}, action) {
    let newState;
    switch(action.type) {
        case 'SOCKET_CONNECTED':
            console.debug('Socket connected, uid =', action.socketID);
            return { socketID: action.socketID, socketState: 'connected', room: null };
        case 'SOCKET_DISCONNECTED':
            console.debug('Socket connection closed');
            return { socketID: null, room: null, socketState: 'disconnected' };
        case 'SOCKET_RECONNECTING':
            console.debug('Attempting to reconnect');
            newState = { ...state };
            newState.socketState = 'reconnecting';
            return newState;
        case 'SOCKET_RECONNECTION_FAILED':
            console.debug('Failed to reconnect');
            newState = { ...state };
            newState.socketState = 'failed';
            return newState;
        case 'JOINED_ROOM':
            console.debug('Joined room', action.roomCode, 'with users', action.users);
            let messages = [{ type: 'initial_join', roomCode: action.roomCode, users: action.users }];
            newState = { ...state };
            newState.room = { users: action.users, code: action.roomCode, messages };
            return newState;
        case 'LEFT_ROOM':
            console.debug('Left room');
            return { socketID: state.socketID };
        case 'CHAT_MESSAGE':
            newState = { ...state };
            newState.room = { ...newState.room };
            const chatMessage = { type: 'chat', user: state.room.users[action.message.userID], content: action.message.content };
            newState.room.messages = pushItem(newState.room.messages, chatMessage);
            return newState;
        case 'WINNER':
            newState = { ...state };
            newState.room = { ...newState.room };
            const winMessage = { type: 'winner', winner: state.room.users[action.winnerID], word: action.word };
            newState.room.messages = pushItem(newState.room.messages, winMessage);
            return newState;
        case 'BECOME_LEADER':
            console.debug('Became leader drawing', action.word);
            newState = { ...state };
            newState.room = { ...newState.room };
            newState.room.word = action.word;
            newState.room.isLeader = true;
            newState.room.leader = state.room.users[state.socketID];
            return newState;
        case 'BECOME_GUESSER':
            console.debug('Became guesser leaderid =', action.leaderID);
            newState = { ...state };
            newState.room = { ...newState.room };
            newState.room.word = null;
            newState.room.isLeader = false;
            newState.room.leader = state.room.users[action.leaderID];
            return newState;
        case 'USER_JOINED':
            newState = { ...state };
            newState.room = { ...newState.room };
            newState.room.users = Object.assign({}, newState.room.users);
            // This should be the same data structure as is created in JOINED_ROOM
            newState.room.users[action.userID] = { username: action.username, id: action.userID };
            const joinMessage = { type: 'user_join', user: newState.room.users[action.userID] };
            newState.room.messages = pushItem(newState.room.messages, joinMessage);

            return newState;
        case 'USER_LEFT':
            newState = { ...state };
            newState.room = { ...newState.room };
            newState.room.users = Object.keys(state.room.users).reduce((acc, key) => {
                // Copy all keys except user key to new object
                if (key !== action.userID) acc[key] = newState.room.users[key];
                return acc;
            }, {});

            const leaveMessage = { type: 'user_left', user: state.room.users[action.userID] };
            newState.room.messages = pushItem(newState.room.messages, leaveMessage);
            return newState;
        case 'LEAVE_ROOM':
            console.debug('Leave room');
            return { socketID: state.socketID }
        default:
            console.warn('Unhandled action in state', action, 'state was:', state);
            return state;
    }
}

export default createStore(stateManager);
