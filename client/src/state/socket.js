import { socketConnected, joinedRoom, becomeLeader, becomeGuesser, chatMessage, userJoinedRoom, userLeftRoom, winner } from './action';

export default class SocketManager {
    constructor(url, store) {
        this.socket = new WebSocket(url);
        this.socket.onmessage = this.handleSocketMessage.bind(this);
        this.store = store;
        this.drawHandler = null;
        this.newRoundHandler = null;
    }

    handleSocketMessage(e) {
        let message = e.data;
        if (message[0] === 'd') {
            let p = message.slice(1).split(',').map(str => parseInt(str));
            if (this.drawHandler)
                this.drawHandler.apply(undefined, p);
        } else if (message[0] === 'c') {
            let id = message.slice(1);
            this.store.dispatch(socketConnected(id))
        } else if (message[0] === 'm') {
            message = message.slice(1).split(',');
            this.store.dispatch(chatMessage(message[0], message[1]));
        } else if (message[0] === 'l') {
            let word = message.slice(1);
            this.store.dispatch(becomeLeader(word));
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
            this.store.dispatch(userLeftRoom());
        } else if (message[0] === 'w') {
            var data = message.slice(1).split(',');
            this.store.dispatch(winner(data[0], data[1]));
        } else {
            console.log(message);
        }
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
}
