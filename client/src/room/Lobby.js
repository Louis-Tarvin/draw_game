import React, { useState, useRef, useEffect } from 'react';
import { useSelector } from 'react-redux';
import classNames from 'classnames';

function Wordpack({ toggleSelected, isSelected, name, description, id }) {
    return (
        <div
            className={classNames('wordpack', { 'selected': isSelected })}
            onClick={toggleSelected.bind(null, id)} >
            <div className="wordpack-name">{name}</div>
            <div className="wordpack-description">{description}</div>
        </div>
    )
}

export default function Lobby({ socketManager }) {
    const [canStart, setCanStart] = useState(false);
    const wordpacks = useSelector(state => state.room.wordpacks);
    const [selectedWordpacks, setSelectedWordpacks] = useState({});
    const roundTimerCheckboxRef = useRef(null);
    const canvasClearCheckboxRef = useRef(null);

    const onStart = e => {
        e.preventDefault();
        const timeLimit = roundTimerCheckboxRef.current.checked? 'T': 'F';
        const canvasClearing = canvasClearCheckboxRef.current.checked? 'T': 'F';
        const selectedIDs = Object.keys(selectedWordpacks).filter(id => selectedWordpacks[id]);
        if (selectedIDs) {
            socketManager.startGame(selectedIDs, timeLimit, canvasClearing);
        }
    }

    const toggleSelected = id => {
        const selected = { ...selectedWordpacks };
        selected[id] = !selected[id];
        setSelectedWordpacks(selected);
    }

    useEffect(() => {
        setCanStart(Object.keys(selectedWordpacks).filter(id => selectedWordpacks[id]).length > 0);
    }, [selectedWordpacks, setCanStart]);

    return (
        <>
            <h2>Room Settings:</h2>
            <form className="start-form" onSubmit={onStart}>
                <label className="checkbox-wrapper" htmlFor="round-timer-checkbox">Round time limit
                    <input type="checkbox" id="round-timer-checkbox" ref={roundTimerCheckboxRef} />
                    <span className="checkbox-span"></span>
                </label>
                <label className="checkbox-wrapper" htmlFor="canvas-clear-checkbox">Allow canvas clearing
                    <input type="checkbox" id="canvas-clear-checkbox" ref={canvasClearCheckboxRef} />
                    <span className="checkbox-span"></span>
                </label>
                <div>
                    <h2>Wordpacks:</h2>
                </div>
                <div className="wordpacks-wrapper">
                    {wordpacks?
                        wordpacks.map(data => (
                            <Wordpack
                                key={data.id}
                                toggleSelected={toggleSelected}
                                isSelected={selectedWordpacks[data.id]}
                                {...data} />)
                            )
                        : 'loading'}
                </div>
                <input type="submit" value="Start Game" id="start-button" disabled={!wordpacks || !canStart} />
            </form>
        </>
    )
}
