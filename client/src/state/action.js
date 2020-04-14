export function socketConnected(socketID) {
    return { type: 'SOCKET_CONNECTED', socketID };
}

export function socketDisconnected() {
    return { type: 'SOCKET_DISCONNECTED' }
}

export function joinedRoom(roomCode, users) {
    return { type: 'JOINED_ROOM', roomCode, users };
}

export function leftRoom() {
    return { type: 'LEFT_ROOM' };
}

export function chatMessage(userID, content) {
    return { type: 'CHAT_MESSAGE', message: { userID, content } };
}

export function winner(winnerID, rawWord) {
    const word = rawWord[0].toUpperCase() + rawWord.slice(1);
    return { type: 'WINNER', winnerID, word };
}

export function becomeLeader(rawWord) {
    const word = rawWord[0].toUpperCase() + rawWord.slice(1);
    return { type: 'BECOME_LEADER', word };
}

export function becomeGuesser(leaderID) {
    return { type: 'BECOME_GUESSER', leaderID };
}

export function userJoinedRoom(userID, username) {
    return { type: 'USER_JOINED', userID, username };
}

export function userLeftRoom(userID) {
    return { type: 'USER_LEFT', userID };
}
