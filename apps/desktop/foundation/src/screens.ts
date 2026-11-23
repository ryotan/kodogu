import type {FoundationScreens} from '@kodogu/desktop-routing/router';

import {PopoverPendingLayout} from '#components/PopoverPendingLayout';
import {AppLayout} from '#screens/AppLayout';
import {PopoverLayout} from '#screens/PopoverLayout';
import {RootLayout} from '#screens/RootLayout';
import {StarterPage} from '#screens/StarterPage';

// The screens of the foundation by route ID. The package that creates the router attaches them.
export const foundationScreens = {
  __root__: {component: RootLayout, pendingComponent: PopoverPendingLayout},
  '/_app': {component: AppLayout},
  '/_app/': {component: StarterPage},
  '/popover': {component: PopoverLayout},
} satisfies FoundationScreens;
