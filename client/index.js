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

        socket.send("d"+[startX, startY, endX, endY, penSize].join(","));
    }

    function enableDrawMode() {
        var drawEnabled = false;
        var penSize = 5;
        var prevX = 0;
        var prevY = 0;

        isLeader = true;

        canvas.addEventListener("mousedown", function(e) {
            drawEnabled = true;
            prevX = e.clientX - rect.left;
            prevY = e.clientY - rect.top;
            line(prevX, prevY, prevX, prevY, penSize);
        });
        document.addEventListener("mouseup", function() {
            drawEnabled = false;
        });
        canvas.addEventListener("mousemove", function(e) {
            if (!drawEnabled) {
                return;
            }
            var x = e.clientX - rect.left;
            var y = e.clientY - rect.top;

            context.strokeStyle = 'black';
            line(x, y, prevX, prevY, penSize);
            prevX = x;
            prevY = y;
        });
    }

    function disableDrawMode() {
        isLeader = false;
        canvas.removeEventListener("mousedown");
        document.removeEventListener("mouseup");
        canvas.removeEventListener("mousemove");
    }

    function enableSpectatorMode() {

    }

    enableDrawMode();
}
