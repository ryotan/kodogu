import React from 'react';
import ReactDOM from 'react-dom/client';

import {RouterProvider, createRouter} from '@tanstack/react-router';

// Import the generated route tree
import {routeTree} from '!#router/routes';

// Create a new router instance
const router = createRouter({routeTree});

// Register the router instance for type safety
declare module '@tanstack/react-router' {
  // eslint-disable-next-line @typescript-eslint/consistent-type-definitions
  interface Register {
    router: typeof router;
  }
}

const root = document.getElementById('root');
if (root && !root.innerHTML) {
  const react = ReactDOM.createRoot(root);
  react.render(
    <React.StrictMode>
      <RouterProvider router={router} />
    </React.StrictMode>,
  );
}
