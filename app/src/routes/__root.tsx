import type {QueryClient} from '@tanstack/react-query';
import {createRootRouteWithContext, Outlet} from '@tanstack/react-router';

import {TanStackTools} from '#(dev)/TanStackTools';
import {DesignTokenSync} from '#components/DesignTokenSync';
import {PopoverPendingLayout} from '#components/PopoverPendingLayout';
import {appConfigQuery} from '#hooks/useAppConfig';

export type RouterContext = {
  queryClient: QueryClient;
};

export const Route = createRootRouteWithContext<RouterContext>()({
  beforeLoad: async ({context: {queryClient}}) => {
    await queryClient.ensureQueryData(appConfigQuery);
  },
  pendingComponent: PopoverPendingLayout,
  component: () => (
    <>
      <DesignTokenSync />
      <Outlet />
      <TanStackTools />
    </>
  ),
});
