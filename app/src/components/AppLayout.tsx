// Layout for the /_app route — provides the Radix Theme that all _app/* pages render inside,
// plus the dev-only Radix theme panel. Lives outside routes/ so the file-based router can
// keep auto code splitting the route module.

import {lazy, Suspense} from 'react';

import {Outlet} from '@tanstack/react-router';

import {Theme} from '@radix-ui/themes';

// Dev-only — excluded from non-dev bundles via Vite dead-code elimination
const RadixThemePanel = __DEV_TOOLS__
  ? lazy(() => import('#(dev)/RadixThemePanel').then(m => ({default: m.RadixThemePanel})))
  : () => null;

export function AppLayout() {
  return (
    <Theme panelBackground='solid'>
      <Outlet />
      <Suspense fallback={null}>
        <RadixThemePanel />
      </Suspense>
    </Theme>
  );
}
