import '@radix-ui/themes/styles.css';
import './main.css';

import {StrictMode} from 'react';
import {createRoot} from 'react-dom/client';

import {QueryClient, QueryClientProvider} from '@tanstack/react-query';
import {createRouter, RouterProvider} from '@tanstack/react-router';

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

const queryClient = new QueryClient();

const root = document.getElementById('root');
if (root && !root.innerHTML) {
  const react = createRoot(root);
  react.render(
    <StrictMode>
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>
    </StrictMode>,
  );
}
