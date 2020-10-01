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

function parseCustomWords(inputText) {
    const text = inputText.trim();
    return text.split('\n').map(s => s.split(',').map(w => w.trim()))
}

function debounce(cb, timeout) {
    let timeout_id = null;
    return [e => {
            if (timeout_id !== null) {
                clearTimeout(timeout_id);
            }
            timeout_id = setTimeout(() => {
                console.log("event",e);
                timeout_id = null;
                cb(e)
            }, timeout);
        },
        () => {
            if (timeout !== null) {
                clearTimeout(timeout_id);
            }
        }]
}

export default function Lobby({ socketManager }) {
    const [canStart, setCanStart] = useState(false);
    const wordpacks = useSelector(state => state.room.wordpacks);
    const [selectedWordpacks, setSelectedWordpacks] = useState({});
    const roundTimerCheckboxRef = useRef(null);
    const canvasClearCheckboxRef = useRef(null);
    const [customWords, setCustomWords] = useState("");
    const [parsedCustomWords, setParsedCustomWords] = useState([]);
    const [parseCallback, setParseCallback] = useState(() => {});

    const onStart = e => {
        e.preventDefault();
        const timeLimit = roundTimerCheckboxRef.current.checked? 'T': 'F';
        const canvasClearing = canvasClearCheckboxRef.current.checked? 'T': 'F';
        const selectedIDs = Object.keys(selectedWordpacks).filter(id => selectedWordpacks[id]);
        const customWordPack = parseCustomWords(customWords).map(words => words.join(',')).join('|');
        if (canStart) {
            socketManager.startGame(selectedIDs, timeLimit, canvasClearing, customWordPack);
        }
    }

    const customWordsChanged = e => {
        parseCallback(e.target);
        setCustomWords(e.target.value);
    }

    const toggleSelected = id => {
        const selected = { ...selectedWordpacks };
        selected[id] = !selected[id];
        setSelectedWordpacks(selected);
    }

    useEffect(() => {
        setCanStart(Object.keys(selectedWordpacks).filter(id => selectedWordpacks[id]).length > 0 || parsedCustomWords.length > 0);
    }, [selectedWordpacks, setCanStart, parsedCustomWords]);

    useEffect(() => {
        const [debouncedParse, cancelParse] = debounce(target => {
            setParsedCustomWords(parseCustomWords(target.value));
        }, 300);
        setParseCallback(() => debouncedParse);
        return cancelParse
    }, [setParsedCustomWords, setParseCallback]);

    return (
        <>
            <h2>Room Settings:</h2>
            <form className="start-form" onSubmit={onStart}>
                <label className="checkbox-wrapper" htmlFor="round-timer-checkbox">Two minute round timer
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
                <div className = "custom-wordpack">
                    <textarea onChange={customWordsChanged}></textarea>
                </div>
                <input type="submit" value="Start Game" id="start-button" disabled={!wordpacks || !canStart} />
            </form>
        </>
    )
}
