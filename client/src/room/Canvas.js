import React, { useEffect, useState, useCallback } from 'react';
import { useSelector } from 'react-redux';

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

function Pen({ penSize, x, y }) {

    const style = {
        width: penSize * 2,
        height: penSize * 2,
        top: y - penSize,
        left: x - penSize
    };

    return (
        <div className="pen" style={style}></div>
    );
}

function PenChanger({ penSize, setPenSize }) {

    return (
        <div className="pen-changer">
            <input
                type="range"
                min="1"
                max="10"
                value={penSize}
                onChange={e => setPenSize(e.target.value)}
            />
        </div>
    );
}

export default function Canvas({ socketManager, isLeader }) {
    const [context, canvas, canvasRef] = useCanvasContext();
    const [penDown, setPenDown] = useState(false);
    const [penLeft, setPenLeft] = useState(false);

    const canvasClearing = useSelector(state => state.room.canvasClearing);

    const [prevX, setPrevX] = useState(0);
    const [prevY, setPrevY] = useState(0);
    const [penSize, setPenSize] = useState(2);

    useEffect(() => {
        if (canvas) {
            canvas.addEventListener('contextmenu', event => event.preventDefault());
        }
    }, [canvas]);

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
            socketManager.sendDraw([startX, startY, endX, endY, penSize]);
        }
    }, [context, socketManager, isLeader]);

    const drawCleanLine = useCallback((startX, startY, endX, endY, penSize) => {
        drawLine.apply(null, [startX, startY, endX, endY, penSize]
            .map(x => Math.round(x))
            .map(x => x < 0? 0: x)
            .map(x => x > 500? 500: x)
        );
    }, [drawLine]);

    const clearCanvas = useCallback(() => {
        if (context)
            context.clearRect(0, 0, canvas.width, canvas.height);
    }, [context, canvas]);

    const eraseCanvas = useCallback(() => {
        socketManager.clear();
        clearCanvas();
    }, [socketManager, clearCanvas]);

    useEffect(() => {
        // Only set the handler when the context is valid
        if (context) {
            socketManager.setDrawHandler((erase, startX, startY, endX, endY, penSize) => {
                if (!isLeader) {
                    if (erase) {
                        clearCanvas();
                    } else {
                        drawCleanLine(startX, startY, endX, endY, penSize);
                    }
                }
            });

            return () => {
                socketManager.setDrawHandler(null);
            }
        }
    }, [drawCleanLine, socketManager, isLeader, clearCanvas, context]);

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
        if (canvas) {
            const rect = canvas.getBoundingClientRect();
            const x = e.clientX - rect.left;
            const y = e.clientY - rect.top;
            if (isLeader && penDown && !penLeft) {
                drawCleanLine(x, y, prevX, prevY, penSize);
            }
            setPrevX(x);
            setPrevY(y);
        }
    }, [isLeader, penDown, canvas, prevX, prevY, penSize, drawCleanLine, penLeft]);

    const mouseDown = useCallback(e => {
        if (isLeader && canvas) {
            setPenDown(true);
            setPenLeft(false);
            const rect = canvas.getBoundingClientRect();
            const x = e.clientX - rect.left;
            const y = e.clientY - rect.top;
            drawCleanLine(x, y, x, y, penSize);

            setPrevX(x);
            setPrevY(y);
        }
    }, [isLeader, canvas, drawCleanLine, penSize]);

    const mouseEnter = useCallback(e => {
        if (isLeader && penDown && canvas) {
            const rect = canvas.getBoundingClientRect();
            const x = e.clientX - rect.left;
            const y = e.clientY - rect.top;
            drawCleanLine(x, y, x, y, penSize);

            setPrevX(x);
            setPrevY(y);

            // Enable normal drawing
            setPenLeft(false);
        }
    }, [isLeader, canvas, drawCleanLine, penSize, penDown]);

    // When the mouse leaves mark it as such and complete the line to the edge
    const mouseLeft = useCallback(e => {
        if (isLeader && penDown && canvas) {
            setPenLeft(true);
            const rect = canvas.getBoundingClientRect();
            const x = e.clientX - rect.left;
            const y = e.clientY - rect.top;

            drawCleanLine(x, y, prevX, prevY, penSize);
        }
    }, [isLeader, penDown, canvas, drawCleanLine, penSize, prevX, prevY]);

    const download = useCallback(e => {
        if (canvas) {
            const a = document.createElement('a');
            a.href = canvas.toDataURL();
            a.download = 'drawing';
            a.click();
        }
    }, [canvas]);

    return (
        <>
            <div className={isLeader? "draw-toolbar": "draw-toolbar hide"}>
                {canvasClearing? <input className="clear-button" type="submit" onClick={eraseCanvas} value="Clear canvas"  />: null}
                <p className="pen-changer-label" >pen size: </p>
                <PenChanger penSize={penSize} setPenSize={setPenSize} />
            </div>
            <div className={isLeader? "canvas-wrapper hide-cursor": "canvas-wrapper"}>
                {isLeader? <Pen {...{ penSize, x: prevX, y: prevY }} />: null}
                <canvas
                    ref={canvasRef}
                    onMouseDown={mouseDown}
                    onMouseMove={mouseMove}
                    onMouseEnter={mouseEnter}
                    onMouseLeave={mouseLeft}
                    width="500"
                    height="500">
                </canvas>
            </div>
            <input className="download-button" type="submit" value="Download drawing" onClick={download} />
        </>
    );
}
