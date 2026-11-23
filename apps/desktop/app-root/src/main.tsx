import '@radix-ui/themes/styles.css';
import '@kodogu/desktop-foundation/main.css';

import {StrictMode} from 'react';
import {createRoot} from 'react-dom/client';

import {QueryClientProvider, QueryErrorResetBoundary} from '@tanstack/react-query';
import {RouterProvider} from '@tanstack/react-router';

import {AppErrorPage} from '@kodogu/desktop-foundation/components/AppErrorPage';
import {PopoverRouteErrorPage} from '@kodogu/desktop-foundation/components/PopoverRouteErrorPage';
import {createQueryClient} from '@kodogu/desktop-foundation/createQueryClient';
import {loadAppConfig} from '@kodogu/desktop-foundation/loaders/loadAppConfig';
import {foundationScreens} from '@kodogu/desktop-foundation/screens';
import {createAppRouter} from '@kodogu/desktop-routing/router';
import {ErrorBoundary} from 'react-error-boundary';

const queryClient = createQueryClient();

// This package integrates the application: it hands the route definitions what their loaders need, and attaches the
// screens of the foundation and of every tool. The argument does not type-check while a route has no screen.
const router = createAppRouter({
  context: {queryClient, loadBeforeFirstScreen: () => loadAppConfig(queryClient)},
  defaultErrorComponent: PopoverRouteErrorPage,
  screens: foundationScreens,
});

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
