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
  beforeLoad: async ({context: {queryClient}}) => {
    await queryClient.ensureQueryData(appConfigQuery);
  },
  pendingComponent: PopoverPendingLayout,
  component: RootLayout,
});
