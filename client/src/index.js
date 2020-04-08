import React from 'react';
import ReactDOM from 'react-dom';
import { Provider } from 'react-redux';
import store from './state/store';
import SocketManager from './state/socket';

import './index.css';
import App from './App';
import * as serviceWorker from './serviceWorker';

let ws_proto = 'ws://';
if (window.location.protocol === 'https:') {
    ws_proto = 'wss://'
}

const socketManager = new SocketManager(ws_proto + window.location.host + '/ws/', store);

ReactDOM.render(
  <React.StrictMode>
    <Provider store={store}>
        <App socketManager={socketManager} />
    </Provider>
  </React.StrictMode>,
  document.getElementById('root')
);

// If you want your app to work offline and load faster, you can change
// unregister() to register() below. Note this comes with some pitfalls.
// Learn more about service workers: https://bit.ly/CRA-PWA
serviceWorker.unregister();
