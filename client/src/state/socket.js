import {
    socketConnected,
    socketDisconnected,
    joinedRoom,
    leftRoom,
    enterLobby,
    receiveSettingsData,
    becomeLeader,
    becomeGuesser,
    chatMessage,
    userJoinedRoom,
    userLeftRoom,
    winner
} from './action';

export default class SocketManager {
    constructor(url, store) {
        this.url = url;
        this.connect();
        this.store = store;
        this.drawHandler = null;
        this.newRoundHandler = null;
    }

    connect() {
        this.socket = new WebSocket(this.url);
        this.socket.onmessage = this.handleSocketMessage.bind(this);
        this.socket.onclose = this.onClose.bind(this);
    }

    handleSocketMessage(e) {
        let message = e.data;
        if (message[0] === 'd') {
            let p = message.slice(1).split(',').map(str => parseInt(str));
            if (this.drawHandler)
                this.drawHandler.apply(undefined, [false, ...p]);
        } else if (message[0] === 'b') {
            if (this.drawHandler) {
                this.drawHandler(true);
            }
        } else if (message[0] === 'c') {
            let id = message.slice(1);
            this.store.dispatch(socketConnected(id))
        } else if (message[0] === 'm') {
            message = message.slice(1).split(',');
            this.store.dispatch(chatMessage(message[0], message[1]));
        } else if (message[0] === 'l') {
            let canvasClearing = message[1] === 'T';
            let word = message.slice(2);
            this.store.dispatch(becomeLeader(canvasClearing, word));
            if(this.newRoundHandler)
                this.newRoundHandler();
        } else if (message[0] === 'r') {
            let leaderID = message.slice(1);
            this.store.dispatch(becomeGuesser(leaderID));
            if(this.newRoundHandler)
                this.newRoundHandler();
        } else if (message[0] === 'e') {
            let parts = message.slice(1).split(',');
            let code = parts.shift();
            let users = {};
            for (var i = 0; i < parts.length; i+= 2) {
                users[parts[i]] = {
                    username: parts[i+1],
                    id: parts[i],
                };
            }
            this.store.dispatch(joinedRoom(code, users))
        } else if (message[0] === 'j') {
            let userJoinParts = message.slice(1).split(',');
            this.store.dispatch(userJoinedRoom(userJoinParts[0], userJoinParts[1]));
        } else if (message[0] === 'g') {
            let userID = message.slice(1);
            this.store.dispatch(userLeftRoom(userID));
        } else if (message[0] === 'q') {
            this.store.dispatch(leftRoom());
        } else if (message[0] === 'w') {
            let data = message.slice(1).split(',');
            this.store.dispatch(winner(data[0], data[1]));
        } else if (message[0] === 'o') {
            let userID = message.slice(1);
            this.store.dispatch(enterLobby(userID));
        } else if (message[0] === 's') {
            let lines = message.split('\n').slice(1);
            let wordpacks = [];
            for (var i = 0; i < lines.length; i++) {
                let wordpack = lines[i].split(',');
                let wordpackID = wordpack[0];
                let wordpackName = wordpack[1];
                let wordpackDescription = wordpack.slice(2).join(',');
                wordpacks.push({ id: wordpackID, name: wordpackName, description: wordpackDescription });
            }
            this.store.dispatch(receiveSettingsData(wordpacks));
        } else {
            console.log(message);
        }
    }

    onClose() {
        this.store.dispatch(socketDisconnected());
        setTimeout(() => {
            this.connect();
        }, 5000)
    }

    setDrawHandler(callback) {
        this.drawHandler = callback;
    }

    setNewRoundHandler(callback) {
        this.newRoundHandler = callback;
    }

    sendChat(message) {
        this.socket.send('m' + message);
    }

    joinRoom(roomCode, username) {
        this.socket.send('j' + roomCode + ',' + username);
    }

    leaveRoom() {
        this.socket.send('q');
    }

    createRoom(username) {
        this.socket.send('n' + username);
    }

    sendDraw(params) {
        this.socket.send('d' + params.join(','));
    }

    clear() {
        this.socket.send('c');
    }

    startGame(selectedWordpackIDs, timeLimit, canvasClearing) {
        this.socket.send(['s', selectedWordpackIDs.join(','), timeLimit, canvasClearing].join('\n'));
    }
}
