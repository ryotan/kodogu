// Renders a route layout component (one that contains <Outlet />) inside a minimal in-memory
// router so stories don't need to set up the full app route tree.

import type {ReactElement} from 'react';

import {createMemoryHistory, createRootRoute, createRouter, RouterProvider} from '@tanstack/react-router';

type LayoutComponent = () => ReactElement | null;

export function renderInRouter(component: LayoutComponent): ReactElement {
  const root = createRootRoute({component});
  const router = createRouter({
    routeTree: root,
    history: createMemoryHistory({initialEntries: ['/']}),
  });
  return <RouterProvider router={router} />;
}
