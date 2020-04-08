import { createStore } from 'redux';

function pushItem(array, item) {
    return [...array, item];
}

function stateManager(state = {}, action) {
    let newState;
    switch(action.type) {
        case 'SOCKET_CONNECTED':
            console.debug('Socket connected, uid =', action.socketID);
            return { socketID: action.socketID }
        case 'JOINED_ROOM':
            console.debug('Joined room', action.roomCode, 'with users', action.users);
            let messages = [{ type: 'initial_join', roomCode: action.roomCode, users: action.users }]
            return {
                socketID: state.socketID,
                room: { users: action.users, code: action.roomCode, messages }
            };
        case 'LEFT_ROOM':
            console.debug('Left room');
            return { socketID: state.socketID };
        case 'CHAT_MESSAGE':
            newState = { ...state };
            newState.room = { ...newState.room };
            const chatMessage = { type: 'chat', user: state.room.users[action.message.userID], content: action.message.content };
            newState.room.messages = pushItem(newState.room.messages, chatMessage);
            return newState;
        case 'winner':
            newState = { ...state };
            newState.room = { ...newState.room };
            const winMessage = { type: 'winner', winner: state.room.users[action.userID], word: action.word };
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
        default:
            console.warn('Unhandled action in state', action, 'state was:', state);
            return state;
    }
}

export default createStore(stateManager);
