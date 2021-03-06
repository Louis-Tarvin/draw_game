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

export function enterLobby(hostID) {
    return { type: 'ENTER_LOBBY', hostID };
}

export function receiveSettingsData(wordpacks) {
    return { type: 'RECEIVE_SETTINGS', wordpacks };
}

export function chatMessage(userID, content) {
    return { type: 'CHAT_MESSAGE', message: { userID, content } };
}

export function winner(winnerID, points, rawWord, rawAlternate) {
    const word = rawWord[0].toUpperCase() + rawWord.slice(1);
    let alternate;
    if (rawAlternate) {
        alternate = rawAlternate[0].toUpperCase() + rawAlternate.slice(1);
    }
    return { type: 'WINNER', winnerID, points, word, alternate };
}

export function timeout(rawWord) {
    const word = rawWord[0].toUpperCase() + rawWord.slice(1);
    return { type: 'TIMEOUT', word };
}

export function becomeLeader(canvasClearing, rawWord, rawTimeout) {
    let timeout;
    if (rawTimeout === '0') {
        timeout = null;
    } else {
        timeout = new Date(Number(rawTimeout));
    }
    const word = rawWord[0].toUpperCase() + rawWord.slice(1);
    return { type: 'BECOME_LEADER', canvasClearing, word, timeout };
}

export function becomeGuesser(leaderID, rawTimeout) {
    let timeout;
    if (rawTimeout === '0') {
        timeout = null;
    } else {
        timeout = new Date(Number(rawTimeout));
    }
    return { type: 'BECOME_GUESSER', leaderID, timeout };
}

export function userJoinedRoom(userID, username) {
    return { type: 'USER_JOINED', userID, username };
}

export function userLeftRoom(userID) {
    return { type: 'USER_LEFT', userID };
}
