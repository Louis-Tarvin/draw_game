class Game {
    constructor(socketURL) {
        this.socketManager = new SocketManager(socketURL, this.handleSocketMessage.bind(this));

        this.interface = new InterfaceManager(this.socketManager, this.drawCallback.bind(this));
        this.isLeader = false;
        this.id;
    }

    handleSocketMessage(e) {
        var message = e.data;
        if (message[0] == 'd') {
            if (!this.isLeader) {
                var p = message.slice(1).split(',').map(function(str) {
                    return parseInt(str);
                });
                this.interface.handleDraw(p);
            }
        } else if (message[0] == 'c') {
            self.id = message.slice(1);
        } else if (message[0] == 'm') {
            message = message.slice(1).split(',');
            this.interface.addChat(message[0], message[1]);
        } else if (message[0] == 'l') {
            var word = message.slice(1);
            this.becomeLeader(word);
        } else if (message[0] == 'r') {
            var username = message.slice(1);
            this.becomeGuesser(username);
        } else if (message[0] == 'e') {
            var parts = message.slice(1).split(',');
            var code = parts.shift();
            var users = {};
            for (var i = 0; i < parts.length; i+= 2) {
                users[parts[i]] = {
                    username: parts[i+1],
                    isCurrentUser: this.id === parts[i],
                };
            }
            this.interface.enteredRoom(code, users);
        } else if (message[0] == 'j') {
            var parts = message.slice(1).split(',');
            this.interface.joinRoom(parts[0], parts[1]);
        } else if (message[0] == 'g') {
            var userId = message.slice(1);
            this.interface.userGone(userId);
        } else if (message[0] == 'q') {
            this.interface.leftRoom();
        } else if (message[0] == 'w') {
            var data = message.slice(1).split(',');
            this.interface.handleWinner(data[0], data[1]);
        } else {
            console.log(message);
        }
    }

    becomeLeader(word) {
        this.isLeader = true;
        this.interface.becomeLeader(word);
    }

    becomeGuesser(leaderUsername) {
        this.isLeader = false;
        this.interface.becomeGuesser(leaderUsername);
    }

    drawCallback(startX, startY, endX, endY, penSize) {
        if (this.isLeader) {
            var params = [startX, startY, endX, endY, penSize].map(function(param) {
                // Don't allow decimal coordinates
                return Math.round(param);
            });
            this.socketManager.sendDraw(params);
        }
    }
}

class InterfaceManager {
    constructor(socketManager, drawCallback) {
        this.canvasManager = new CanvasManager(drawCallback);

        this.infoBox = document.getElementById('info');
        this.messageBox = document.getElementById('messages');
        this.main = document.getElementById('main');
        this.users = null;

        document.getElementById('username-form').addEventListener('submit', e => {
            e.preventDefault();
            var input = e.target['username'];
            var username = input.value;
            if (username.includes(',') || username.length > 15) {
                alert('username cannot contain a comma and must be less then 15 characters');
            } else if (username == "") {
                alert('username cannot be empty');
            } else {
                this.username = username;
                input.value = '';
                document.getElementById('username-wrapper').classList.add('hide');
                document.getElementById('manage-room').classList.remove('hide');
            }
        });

        document.getElementById('new-room').addEventListener('click', () => {
            socketManager.newRoom(this.username);
        });

        document.getElementById('join-room-form').addEventListener('submit', e => {
            var input = e.target['room-key-input'];
            socketManager.joinRoom(input.value, this.username);
            input.value = '';

            e.preventDefault();
        });

        document.getElementById('leave-room').addEventListener('click', () => {
            socketManager.leaveRoom();
        });

        document.getElementById('chat-form').addEventListener('submit', function(e) {
            e.preventDefault();
            var input = document.getElementById('chat-input');
            socketManager.sendChat(input.value);
            input.value = '';
        })
    }

    info(msg) {
        this.infoBox.textContent = msg;
    }

    addChat(userId, content) {
        var usernameElement = document.createElement('span');
        usernameElement.className = 'username';
        usernameElement.appendChild(document.createTextNode(this.users[userId].username));

        var contentElement = document.createElement('span');
        contentElement.className = 'content';
        contentElement.appendChild(document.createTextNode(content));

        var ele = document.createElement('div');
        ele.appendChild(usernameElement);
        ele.appendChild(contentElement);
        this.messageBox.appendChild(ele);
    }

    printAnnouncement(message) {
        var ele = document.createElement('div');
        ele.className = 'announcement';
        ele.appendChild(document.createTextNode(message));
        this.messageBox.appendChild(ele);
    }

    becomeLeader(word) {
        this.info('Draw: ' + word);
        this.canvasManager.reset();
        this.canvasManager.setEnabled(true);
    }

    becomeGuesser(leaderId) {
        this.info('Guess what ' + this.users[leaderId].username + ' is drawing');
        this.canvasManager.reset();
        this.canvasManager.setEnabled(false);
    }

    enteredRoom(code, users) {
        this.users = users;
        console.debug('Joined room', code);
        this.printAnnouncement('Welcome to room: ' + code);
        var userString = Object.values(this.users).map(user => user.username).join(', ')
        this.printAnnouncement('Current connected users: ' + userString);

        this.main.classList.add('inRoom');

    }

    joinRoom(session_id, username) {
        console.debug(username, 'joined room with session_id', session_id);
        this.printAnnouncement('User ' + username + ' has entered the room');
        this.users[session_id] = { username, isCurrentUser: false };
    }

    leftRoom() {
        console.debug('Left room');
        this.canvasManager.clear();
        this.infoBox.textContent = null;
        this.messageBox.textContent = null;

        this.main.classList.remove('inRoom');
    }

    userGone(session_id) {
        this.printAnnouncement('User ' + this.users[session_id].username + ' has left the room');
        delete this.users[session_id];
    }

    handleWinner(userId, word) {
        this.printAnnouncement(this.users[userId].username
            + ' correctly guessed the word ' + word);
    }

    handleDraw(params) {
        this.canvasManager.line.apply(this.canvasManager, params)
    }
}

class CanvasManager {
    constructor(drawCallback) {
        this.enabled = false;
        this.penActive = false;
        this.left = false;
        this.drawCallback = drawCallback;
        this.canvas = document.getElementById('canvas');
        this.context = this.canvas.getContext('2d');

        this.penSize = 3;
        this.prevX = 0;
        this.prevY = 0;

        this.canvas.addEventListener('mousedown', e => {
            if (this.enabled) {
                this.penActive = true;
                var rect = this.canvas.getBoundingClientRect();
                this.prevX = e.clientX - rect.left;
                this.prevY = e.clientY - rect.top;
                this.line(this.prevX, this.prevY, this.prevX, this.prevY, this.penSize);
            }
        });
        document.addEventListener('mouseup', () => {
            this.penActive = false;
        });
        this.canvas.addEventListener('mousemove', e => {
            if (!(this.penActive && this.enabled)) {
                return;
            }
            var rect = this.canvas.getBoundingClientRect();
            var x = e.clientX - rect.left;
            var y = e.clientY - rect.top;

            this.context.strokeStyle = 'black';
            this.line(x, y, this.prevX, this.prevY, this.penSize);
            this.prevX = x;
            this.prevY = y;
        });
        this.canvas.addEventListener('mouseleave', () => {
            // Prevent lines from going between where the mouse left the canvas and entered
            if (this.penActive) {
                this.left = true;
                this.penActive = false;
            }
        });
        this.canvas.addEventListener('mouseenter', e => {
            // When the mouse re-enters while it is drawing (mouse is down)
            // it should continue drawing from the new position
            if (this.left) {
                this.left = false;
                this.penActive = true;

                var rect = this.canvas.getBoundingClientRect();
                this.prevX = e.clientX - rect.left;
                this.prevY = e.clientY - rect.top;
            }
        });
    }

    setEnabled(enabled) {
        this.enabled = enabled;
    }

    line(startX, startY, endX, endY, penSize) {
        var context = this.context;
        context.beginPath();
        context.ellipse(startX, startY, penSize, penSize, 0, 0, 2 * Math.PI);
        context.fill();

        context.beginPath();
        context.lineWidth = penSize * 2;
        context.moveTo(endX, endY);
        context.lineTo(startX, startY);
        context.stroke();
        context.closePath();

        this.drawCallback(startX, startY, endX, endY, penSize);
    }

    reset() {
        this.clear();
        this.penActive = false;
        this.left = false;
    }

    clear() {
        this.context.clearRect(0, 0, this.canvas.width, this.canvas.height);
    }
}

class SocketManager {
    constructor(socketURL, messageHandler) {
        this.socket = new WebSocket(socketURL);
        this.socket.onmessage = messageHandler;
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

    newRoom(username) {
        this.socket.send('n' + username);
    }

    sendDraw(params) {
        this.socket.send('d' + params.join(','));
    }
}

var game;

window.onload = function() {
    game = new Game('ws://' + window.location.host + '/ws/');
};
