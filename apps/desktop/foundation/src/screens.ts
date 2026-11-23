import {lazyRouteComponent} from '@tanstack/react-router';

import type {FoundationScreens} from '@kodogu/desktop-routing/router';

import {PopoverPendingLayout} from '#components/PopoverPendingLayout';
import {RootLayout} from '#screens/RootLayout';

// The screens of the foundation by route ID. The package that creates the router attaches them. Each screen but the
// root's is loaded when its route is first shown, as the router's automatic code splitting did while the screens were
// in the route files; it splits only components that a route file names.
export const foundationScreens = {
  __root__: {component: RootLayout, pendingComponent: PopoverPendingLayout},
  '/_app': {component: lazyRouteComponent(() => import('#screens/AppLayout'), 'AppLayout')},
  '/_app/': {component: lazyRouteComponent(() => import('#screens/StarterPage'), 'StarterPage')},
  '/popover': {component: lazyRouteComponent(() => import('#screens/PopoverLayout'), 'PopoverLayout')},
} satisfies FoundationScreens;
