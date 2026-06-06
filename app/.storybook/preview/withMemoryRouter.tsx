import {createMemoryHistory, createRootRoute, createRouter, RouterProvider} from '@tanstack/react-router';

import type {Decorator} from '@storybook/react-vite';

declare module '@storybook/react-vite' {
  // biome-ignore lint/style/useConsistentTypeDefinitions: TypeScript module augmentation requires `interface`; `type` cannot extend an existing interface from another module.
  interface Parameters {
    /**
     * Story configuration consumed by the {@link withMemoryRouter} decorator.
     * Has no effect when the decorator is not applied to the story.
     */
    router?: {
      /** Render the story without wrapping it in a memory-history RouterProvider. */
      skip?: boolean;
    };
  }
}

// Wraps a Story in a TanStack Router driven by an in-memory history so route layouts that contain
// <Outlet /> can render without the full app route tree. Apply at meta level on stories whose
// component depends on a router context.
export const withMemoryRouter: Decorator = (Story, {parameters}) => {
  if (parameters.router?.skip === true) {
    return <Story />;
  }
  const router = createRouter({
    routeTree: createRootRoute({component: () => <Story />}),
    history: createMemoryHistory({initialEntries: ['/']}),
    // Layout stories render <Outlet /> but configure no child route. Without this, TanStack Router
    // warns about a missing `notFoundComponent` whenever the Outlet hits a non-existent route.
    defaultNotFoundComponent: () => null,
  });
  return <RouterProvider router={router} />;
};
