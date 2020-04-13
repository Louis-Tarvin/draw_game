import React from 'react';

export default function Lobby({ socketManager }) {

    const onStart = e => {
        e.preventDefault();

        socketManager.startGame();
    }

    return (
        <>
            <form className="start-form" onSubmit={onStart}>
                <input type="submit" value="Start Game" />
            </form>
        </>
    )
}
