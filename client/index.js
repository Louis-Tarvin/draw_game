class Game {
    constructor(socketURL) {
        this.socketManager = new SocketManager(socketURL, this.handleSocketMessage.bind(this));

        this.interface = new InterfaceManager(this.socketManager, this.drawCallback.bind(this));
        this.isLeader = false;
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
            message = message.slice(1).split(',');
            this.interface.addChat(message[0], message[1]);
        } else if (message[0] == 'l') {
            var word = message.slice(1);
            this.becomeLeader(word);
        } else if (message[0] == 'r') {
            var username = message.slice(1);
            this.becomeGuesser(username);
        } else if (message[0] == 'j') {
            var code = message.slice(1);
            this.interface.joinedRoom(code);
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

        document.getElementById('new-room').addEventListener('click', () => {
            socketManager.newRoom();
        });

        document.getElementById('join-room-form').addEventListener('submit', e => {
            var input = e.target['room-key-input'];
            socketManager.joinRoom(input.value);
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

    addChat(username, content) {
        var usernameElement = document.createElement('span');
        usernameElement.className = 'username';
        usernameElement.appendChild(document.createTextNode(username));

        var contentElement = document.createElement('span');
        contentElement.className = 'content';
        contentElement.appendChild(document.createTextNode(content));

        var ele = document.createElement('div');
        ele.appendChild(usernameElement);
        ele.appendChild(contentElement);
        this.messageBox.appendChild(ele);
    }

    becomeLeader(word) {
        this.info('Draw: ' + word);
        this.canvasManager.reset();
        this.canvasManager.setEnabled(true);
    }

    becomeGuesser(leaderUsername) {
        this.info('Guess what ' + leaderUsername + ' is drawing');
        this.canvasManager.reset();
        this.canvasManager.setEnabled(false);
    }

    joinedRoom(code) {
        console.debug('Joined room', code);
        var welcome_message = document.createElement('span');
        welcome_message.appendChild(document.createTextNode('Welcome to room: ' + code))
        var ele = document.createElement('div');
        ele.appendChild(welcome_message);
        this.messageBox.appendChild(ele);

        this.main.classList.add('inRoom');
    }

    leftRoom() {
        console.debug('Left room');
        this.canvasManager.clear();
        this.infoBox.textContent = null;
        this.messageBox.textContent = null;

        this.main.classList.remove('inRoom');
    }

    handleWinner(username, word) {
        var win_message = document.createElement('span');
        win_message.appendChild(document.createTextNode(username + ' correctly guessed the word ' + word))
        var ele = document.createElement('div');
        ele.appendChild(win_message);
        this.messageBox.appendChild(ele);
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
        this.socket.send('c' + message);
    }

    joinRoom(roomCode) {
        this.socket.send('j' + roomCode);
    }

    leaveRoom() {
        this.socket.send('q');
    }

    newRoom() {
        this.socket.send('n');
    }

    sendDraw(params) {
        this.socket.send('d' + params.join(','));
    }
}

var game;

window.onload = function() {
    game = new Game('ws://' + window.location.host + '/ws/');
};
