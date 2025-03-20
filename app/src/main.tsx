import React from 'react';
import ReactDOM from 'react-dom/client';

import App from './App';

const root = document.getElementById('root');
if (root && !root.innerHTML) {
  const react = ReactDOM.createRoot(root);
  react.render(
    <React.StrictMode>
      <App />
    </React.StrictMode>,
  );
}
