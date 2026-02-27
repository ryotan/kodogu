import {createRootRoute, Outlet} from '@tanstack/react-router';

import {TanStackTools} from '#(dev)/TanStackTools';

export const Route = createRootRoute({
  component: () => (
    <>
      <Outlet />
      <TanStackTools />
    </>
  ),
});
