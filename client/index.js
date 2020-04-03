var socket;
var isLeader = false;

window.onload = function() {
    var canvas = document.getElementById("canvas");
    var context = canvas.getContext("2d");
    socket = new WebSocket("ws://localhost:3001/ws/");
    var rect = canvas.getBoundingClientRect();

    socket.addEventListener("message", function(e) {
        var message = e.data;
        if (message[0] == 'd') {
            if (!isLeader) {
                var p = message.slice(1).split(",").map(function(str) {
                    return parseInt(str);
                });
                line.apply(null, p);
            }
        } else if (message[0] == 'c') {
            message = message.slice(1).split(",");
            var username = document.createElement("span");
            username.className = "username";
            username.appendChild(document.createTextNode(message[0]));
            var content = document.createElement("span");
            content.className = "content";
            content.appendChild(document.createTextNode(message[1]));

            var ele = document.createElement("div");
            ele.appendChild(username);
            ele.appendChild(content);
            document.getElementById("messages").appendChild(ele);
        } else if (message[0] == 'l') {
            message = message.slice(1);
            console.log(message);
            isLeader = true;
            context.clearRect(0, 0, canvas.width, canvas.height);
        } else if (message[0] == 'r') {
            isLeader = false;
            context.clearRect(0, 0, canvas.width, canvas.height);
        } else if (message[0] == 'j') {
            code = message.slice(1);
            var welcome_message = document.createElement("span");
            welcome_message.appendChild(document.createTextNode("Welcome to room: "+code))
            var ele = document.createElement("div");
            ele.appendChild(welcome_message);
            document.getElementById("messages").appendChild(ele);
        } else {
            console.log(message);
        }
    });

    function line(startX, startY, endX, endY, penSize) {
        context.beginPath();
        context.ellipse(startX, startY, penSize, penSize, 0, 0, 2*Math.PI);
        context.fill();

        context.beginPath();
        context.lineWidth = penSize * 2;
        context.moveTo(endX, endY);
        context.lineTo(startX, startY);
        context.stroke();
        context.closePath();

        if (isLeader) {
            socket.send("d"+[startX, startY, endX, endY, penSize].join(","));
        }
    }

    function startDrawHandler() {
        var drawEnabled = false;
        var penSize = 5;
        var prevX = 0;
        var prevY = 0;

        canvas.addEventListener("mousedown", function(e) {
            if (isLeader) {
                drawEnabled = true;
                rect = canvas.getBoundingClientRect();
                prevX = e.clientX - rect.left;
                prevY = e.clientY - rect.top;
                line(prevX, prevY, prevX, prevY, penSize);
            }
        });
        document.addEventListener("mouseup", function() {
            if (isLeader) {
                drawEnabled = false;
            }
        });
        canvas.addEventListener("mousemove", function(e) {
            if (!(drawEnabled && isLeader)) {
                return;
            }
            rect = canvas.getBoundingClientRect();
            var x = e.clientX - rect.left;
            var y = e.clientY - rect.top;

            context.strokeStyle = 'black';
            line(x, y, prevX, prevY, penSize);
            prevX = x;
            prevY = y;
        });
    }
    startDrawHandler();

    document.getElementById("new-room").addEventListener("click", function() {
        socket.send("n");
    });

    document.getElementById("join-room").addEventListener("click", function() {
        socket.send("j"+document.getElementById("room-key-input").value);
    });

    document.getElementById("chat-form").addEventListener("submit", function(e) {
        e.preventDefault();
        socket.send("c"+document.getElementById("chat-input").value);
    })
}
