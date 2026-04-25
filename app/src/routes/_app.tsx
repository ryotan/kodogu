import {lazy, Suspense} from 'react';

import {createFileRoute, Outlet} from '@tanstack/react-router';

import {Theme} from '@radix-ui/themes';

// Dev-only — excluded from production bundle via Vite dead-code elimination
const RadixThemePanel = __DEV_TOOLS__
  ? lazy(() => import('#(dev)/RadixThemePanel').then(m => ({default: m.RadixThemePanel})))
  : () => null;

export const Route = createFileRoute('/_app')({
  component: () => (
    <Theme panelBackground='solid'>
      <Outlet />
      <Suspense fallback={null}>
        <RadixThemePanel />
      </Suspense>
    </Theme>
  ),
});
