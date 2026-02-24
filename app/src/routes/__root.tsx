import {createRootRoute, Outlet} from '@tanstack/react-router';

import {Theme} from '@radix-ui/themes';

import {RadixThemePanel} from '#(dev)/RadixThemePanel';
import {TanStackTools} from '#(dev)/TanStackTools';

export const Route = createRootRoute({
  component: () => (
    <>
      <Theme panelBackground='solid'>
        <Outlet />
        <RadixThemePanel />
      </Theme>
      <TanStackTools />
    </>
  ),
});
