import type {QueryClient} from '@tanstack/react-query';
import {createRootRouteWithContext, Outlet} from '@tanstack/react-router';

import {StackTraceInspector} from '#(dev)/StackTraceInspector';
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
      <StackTraceInspector />
    </>
  );
};

export const Route = createRootRouteWithContext<RouterContext>()({
  loader: ({context: {queryClient}}) => queryClient.ensureQueryData(appConfigQuery),
  pendingComponent: PopoverPendingLayout,
  component: RootLayout,
});
