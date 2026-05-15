import '@radix-ui/themes/styles.css';
import './main.css';

import {StrictMode} from 'react';
import {createRoot} from 'react-dom/client';

import {QueryClient, QueryClientProvider} from '@tanstack/react-query';
import {createRouter, RouterProvider} from '@tanstack/react-router';

import {PopoverAppErrorPage} from '#features/menu-popover/components/PopoverAppErrorPage';
import {PopoverErrorBoundary} from '#features/menu-popover/components/PopoverErrorBoundary';

// Import the generated route tree
import {routeTree} from '!#router/routes';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      // Tauri IPC is local — browser online/offline events are irrelevant.
      refetchOnWindowFocus: false,
      refetchOnReconnect: false,
    },
  },
});

// Create a new router instance with queryClient context for route-level data loading.
const router = createRouter({
  routeTree,
  context: {queryClient},
  defaultErrorComponent: PopoverAppErrorPage,
});

// Register the router instance for type safety
declare module '@tanstack/react-router' {
  // biome-ignore lint/style/useConsistentTypeDefinitions: TypeScript module augmentation requires `interface`; `type` cannot extend an existing interface from another module.
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
      <PopoverErrorBoundary>
        <RouterProvider router={router} />
      </PopoverErrorBoundary>
    </QueryClientProvider>
  </StrictMode>,
);
