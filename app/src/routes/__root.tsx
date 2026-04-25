import {lazy, Suspense} from 'react';

import type {QueryClient} from '@tanstack/react-query';
import {createRootRouteWithContext, Outlet} from '@tanstack/react-router';

import {PopoverPendingLayout} from '#components/PopoverPendingLayout';
import {useConfiguredDesignTokens} from '#components/useDesignTokenSync.tsx';
import {appConfigQuery} from '#hooks/useAppConfig';

// Dev-only — excluded from production bundle via Vite dead-code elimination
const TanStackTools = __DEV_TOOLS__
  ? lazy(() => import('#(dev)/TanStackTools').then(m => ({default: m.TanStackTools})))
  : () => null;

export type RouterContext = {
  queryClient: QueryClient;
};

export const RootLayout = () => {
  useConfiguredDesignTokens();
  return (
    <>
      <Outlet />
      <Suspense fallback={null}>
        <TanStackTools />
      </Suspense>
    </>
  );
};

export const Route = createRootRouteWithContext<RouterContext>()({
  loader: ({context: {queryClient}}) => queryClient.ensureQueryData(appConfigQuery),
  pendingComponent: PopoverPendingLayout,
  component: RootLayout,
});
