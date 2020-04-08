import React, { useEffect, useState, useCallback } from 'react';

function useCanvasContext() {
    const [context, setContext] = useState(null);
    const [canvas, setCanvas] = useState(null);
    const canvasRef = useCallback(canvasNode => {
        setCanvas(canvasNode);
        if (canvasNode !== null) {
            setContext(canvasNode.getContext('2d'));
        } else {
            setContext(null);
        }
    }, []);

    return [context, canvas, canvasRef];
}

export default function Canvas({ socketManager, isLeader }) {
    const [context, canvas, canvasRef] = useCanvasContext();
    const [penDown, setPenDown] = useState(false);
    const [penLeft, setPenLeft] = useState(false);

    const [prevX, setPrevX] = useState(0);
    const [prevY, setPrevY] = useState(0);
    // eslint-disable-next-line
    const [penSize, setPenSize] = useState(3);

    const drawLine = useCallback((startX, startY, endX, endY, penSize) => {
        if (!context) {
            console.error('Context wasn\'t available during line drawing');
            return;
        }

        context.beginPath();
        context.ellipse(startX, startY, penSize, penSize, 0, 0, 2 * Math.PI);
        context.fill();

        context.beginPath();
        context.lineWidth = penSize * 2;
        context.moveTo(endX, endY);
        context.lineTo(startX, startY);
        context.stroke();
        context.closePath();

        if (isLeader) {
            socketManager.sendDraw([startX, startY, endX, endY, penSize].map(x => Math.round(x)));
        }
    }, [context, socketManager, isLeader]);

    const clearCanvas = useCallback(() => {
        if (context)
            context.clearRect(0, 0, canvas.width, canvas.height);
    }, [context, canvas]);

    useEffect(() => {
        socketManager.setDrawHandler((startX, startY, endX, endY, penSize) => {
            if (!isLeader) {
                drawLine(startX, startY, endX, endY, penSize);
            }
        });

        return () => {
            socketManager.setDrawHandler(null);
        }
    }, [drawLine, socketManager, isLeader]);

    // Consider whether this is the correct control flow, feels a bit hacky
    useEffect(() => {
        socketManager.setNewRoundHandler(clearCanvas);

        return () => {
            socketManager.setNewRoundHandler(clearCanvas);
        }
    }, [clearCanvas, socketManager]);

    useEffect(() => {
        const listener = () => { setPenDown(false); };
        document.addEventListener("mouseup", listener);
        return () => { document.removeEventListener("mouseup", listener); }
    }, [setPenDown]);

    const mouseMove = useCallback(e => {
        if (isLeader && penDown && !penLeft && canvas) {
            const rect = canvas.getBoundingClientRect();
            const x = e.clientX - rect.left;
            const y = e.clientY - rect.top;

            drawLine(x, y, prevX, prevY, penSize);

            setPrevX(x);
            setPrevY(y);
        }
    }, [isLeader, penDown, canvas, prevX, prevY, penSize, drawLine, penLeft]);

    const mouseDown = useCallback(e => {
        if (isLeader && canvas) {
            setPenDown(true);
            setPenLeft(false);
            const rect = canvas.getBoundingClientRect();
            const x = e.clientX - rect.left;
            const y = e.clientY - rect.top;
            drawLine(x, y, x, y, penSize);

            setPrevX(x);
            setPrevY(y);
        }
    }, [isLeader, canvas, drawLine, penSize]);

    const mouseEnter = useCallback(e => {
        if (isLeader && penDown && canvas) {
            const rect = canvas.getBoundingClientRect();
            const x = e.clientX - rect.left;
            const y = e.clientY - rect.top;
            drawLine(x, y, x, y, penSize);

            setPrevX(x);
            setPrevY(y);

            // Enable normal drawing
            setPenLeft(false);
        }
    }, [isLeader, canvas, drawLine, penSize, penDown]);

    // When the mouse leaves mark it as such and complete the line to the edge
    const mouseLeft = useCallback(e => {
        if (isLeader && penDown && canvas) {
            setPenLeft(true);
            const rect = canvas.getBoundingClientRect();
            const x = e.clientX - rect.left;
            const y = e.clientY - rect.top;

            drawLine(x, y, prevX, prevY, penSize);
        }
    }, [isLeader, penDown, canvas, drawLine, penSize, prevX, prevY]);

    return (
        <canvas
            ref={canvasRef}
            onMouseDown={mouseDown}
            onMouseMove={mouseMove}
            onMouseEnter={mouseEnter}
            onMouseLeave={mouseLeft}
            width="500"
            height="500">
        </canvas>
    );
}
