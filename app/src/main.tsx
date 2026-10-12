import '@radix-ui/themes/styles.css';
import './main.css';

import {StrictMode} from 'react';
import {createRoot} from 'react-dom/client';

import {QueryClientProvider, QueryErrorResetBoundary} from '@tanstack/react-query';
import {createRouter, RouterProvider} from '@tanstack/react-router';

import {ErrorBoundary} from 'react-error-boundary';

import {AppErrorPage} from '#components/AppErrorPage';
import {PopoverRouteErrorPage} from '#components/PopoverRouteErrorPage';
import {createQueryClient} from '#createQueryClient';

// Import the generated route tree
import {routeTree} from '!#router/routes';

const queryClient = createQueryClient();

// Create a new router instance with queryClient context for route-level data loading.
const router = createRouter({
  routeTree,
  context: {queryClient},
  defaultErrorComponent: PopoverRouteErrorPage,
});

// Register the router instance for type safety
declare module '@tanstack/react-router' {
  // eslint-disable-next-line @typescript-eslint/consistent-type-definitions
  interface Register {
    router: typeof router;
  }
}

const root = document.getElementById('root');
if (!root) {
  throw new Error('#root element not found in index.html');
}
createRoot(root).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <QueryErrorResetBoundary>
        {({reset}) => (
          <ErrorBoundary
            fallbackRender={({error, resetErrorBoundary}) => (
              <AppErrorPage
                error={error instanceof Error ? error : new Error(String(error))}
                reset={resetErrorBoundary}
              />
            )}
            onReset={reset}
          >
            <RouterProvider router={router} />
          </ErrorBoundary>
        )}
      </QueryErrorResetBoundary>
    </QueryClientProvider>
  </StrictMode>,
);
