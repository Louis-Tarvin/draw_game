import React, { useState } from 'react';
import { useSelector } from 'react-redux';
import classNames from 'classnames';

function Wordpack({ toggleSelected, isSelected, name, description, id }) {
    return (
        <div
            className={classNames('wordpack', { 'selected': isSelected })}
            onClick={toggleSelected.bind(null, id)} >
            <span>{name}</span>
            <span>{description}</span>
        </div>
    )
}

export default function Lobby({ socketManager }) {
    const wordpacks = useSelector(state => state.room.wordpacks);
    const [selectedWordpacks, setSelectedWordpacks] = useState({});

    const onStart = e => {
        e.preventDefault();
        const selectedIDs = Object.keys(selectedWordpacks).filter(id => selectedWordpacks[id]);
        socketManager.startGame(selectedIDs);
    }

    const toggleSelected = id => {
        const selected = { ...selectedWordpacks };
        selected[id] = !selected[id];
        setSelectedWordpacks(selected);
    }

    return (
        <>
            <h2>Room Settings:</h2>
            <form className="start-form" onSubmit={onStart}>
                <label className="checkbox-wrapper" htmlFor="round-timer-checkbox">Round time limmit
                    <input type="checkbox" id="round-timer-checkbox" />
                    <span className="checkbox-span"></span>
                </label>
                <label className="checkbox-wrapper" htmlFor="canvas-clear-checkbox">Allow canvas clearing
                    <input type="checkbox" id="canvas-clear-checkbox" />
                    <span className="checkbox-span"></span>
                </label>
                <h2>Wordpacks:</h2>
                {wordpacks?
                    wordpacks.map(data => (
                        <Wordpack
                            key={data.id}
                            toggleSelected={toggleSelected}
                            isSelected={selectedWordpacks[data.id]}
                            {...data} />)
                        )
                    : 'loading'}
                <input type="submit" value="Start Game" disabled={!wordpacks} />
            </form>
        </>
    )
}
