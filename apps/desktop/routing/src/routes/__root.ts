import type {QueryClient} from '@tanstack/react-query';
import {createRootRouteWithContext} from '@tanstack/react-router';

export type RouterContext = {
  queryClient: QueryClient;
  // Runs before the first screen is shown. The package that creates the router supplies it, so that the route
  // definitions import no function of the packages that hold screens.
  loadBeforeFirstScreen: () => Promise<unknown>;
};

export const Route = createRootRouteWithContext<RouterContext>()({
  loader: ({context}) => context.loadBeforeFirstScreen(),
});
