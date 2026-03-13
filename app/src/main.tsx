import '@radix-ui/themes/styles.css';
import './main.css';

import {StrictMode} from 'react';
import {createRoot} from 'react-dom/client';

import {QueryClient, QueryClientProvider, QueryErrorResetBoundary} from '@tanstack/react-query';
import {createRouter, RouterProvider} from '@tanstack/react-router';

import {AppBootstrap} from '#components/AppBootstrap';
import {AppErrorPage} from '#components/AppErrorPage';

// Import the generated route tree
import {routeTree} from '!#router/routes';

// Create a new router instance
const router = createRouter({
  routeTree,
  defaultErrorComponent: AppErrorPage,
});

// Register the router instance for type safety
declare module '@tanstack/react-router' {
  // eslint-disable-next-line @typescript-eslint/consistent-type-definitions
  interface Register {
    router: typeof router;
  }
}

const queryClient = new QueryClient();

const root = document.getElementById('root');
if (root) {
  createRoot(root).render(
    <StrictMode>
      <QueryClientProvider client={queryClient}>
        <QueryErrorResetBoundary>
          <AppBootstrap>
            <RouterProvider router={router} />
          </AppBootstrap>
        </QueryErrorResetBoundary>
      </QueryClientProvider>
    </StrictMode>,
  );
}
