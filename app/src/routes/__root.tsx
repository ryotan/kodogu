import type {ReactElement} from 'react';

import type {QueryClient} from '@tanstack/react-query';
import {createRootRouteWithContext, Outlet} from '@tanstack/react-router';

import {TanStackTools} from '#(dev)/TanStackTools';
import {PopoverPendingLayout} from '#components/PopoverPendingLayout';
import {useConfiguredDesignTokens} from '#components/useConfiguredDesignTokens';
import {appConfigQuery} from '#hooks/useAppConfig';

export type RouterContext = {
  queryClient: QueryClient;
};

export function RootLayout(): ReactElement {
  useConfiguredDesignTokens();
  return (
    <>
      <Outlet />
      <TanStackTools />
    </>
  );
}

export const Route = createRootRouteWithContext<RouterContext>()({
  // Keep confirmed settings through route invalidation. A failed query needs a fresh native read.
  loader: ({context: {queryClient}}) =>
    queryClient.query({
      ...appConfigQuery,
      staleTime: queryClient.getQueryState(appConfigQuery.queryKey)?.error ? 0 : 'static',
    }),
  pendingComponent: PopoverPendingLayout,
  component: RootLayout,
});
