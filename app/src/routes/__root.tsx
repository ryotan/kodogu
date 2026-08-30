import type {QueryClient} from '@tanstack/react-query';
import {createRootRouteWithContext, Outlet} from '@tanstack/react-router';

import {TanStackTools} from '#(dev)/TanStackTools';
import {PopoverPendingLayout} from '#components/PopoverPendingLayout';
import {useConfiguredDesignTokens} from '#components/useDesignTokenSync.tsx';
import {appConfigQuery} from '#hooks/useAppConfig';

export type RouterContext = {
  queryClient: QueryClient;
};

export const RootLayout = () => {
  useConfiguredDesignTokens();
  return (
    <>
      <Outlet />
      <TanStackTools />
    </>
  );
};

export const Route = createRootRouteWithContext<RouterContext>()({
  // `staleTime: 'static'` is load-bearing: it is the only staleTime `isStaleByTime()` short-circuits on, so the
  // loader fetches solely while the cache is empty. A numeric one, `Infinity` included, yields to `isInvalidated`.
  loader: ({context: {queryClient}}) => queryClient.query({...appConfigQuery, staleTime: 'static'}),
  pendingComponent: PopoverPendingLayout,
  component: RootLayout,
});
