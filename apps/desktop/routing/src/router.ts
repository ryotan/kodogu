import type {AnyRoute, ErrorRouteComponent, RouteComponent} from '@tanstack/react-router';
import {createRouter} from '@tanstack/react-router';

import {routeTree} from '../gen/router/routes.ts';

import type {RouterContext} from './routes/__root.ts';

export type {RouterContext} from './routes/__root.ts';

// This package holds the route definitions and the router's type, and no screen. The router's type is registered
// here, in the package that every package with screens depends on.
function createBareRouter(context: RouterContext, defaultErrorComponent: ErrorRouteComponent) {
  return createRouter({routeTree, context, defaultErrorComponent});
}

export type AppRouter = ReturnType<typeof createBareRouter>;

declare module '@tanstack/react-router' {
  // eslint-disable-next-line @typescript-eslint/consistent-type-definitions -- Register requires declaration merging.
  interface Register {
    router: AppRouter;
  }
}

export type RouteId = keyof AppRouter['routesById'];

type UpdatableOptions<TId extends RouteId> = Parameters<AppRouter['routesById'][TId]['update']>[0];

// One entry, with a screen, for every route in TId.
// `never` when the route IDs are not known: a package that reads this package's declarations sees the router as `any`
// if a module that the declarations name cannot be resolved from there, and no map satisfies `never`.
export type ScreenMap<TId extends RouteId> = string extends RouteId
  ? never
  : {[K in TId]: UpdatableOptions<K> & {component: RouteComponent}};

type ToolRouteId = Extract<RouteId, `/(tools)/${string}`>;

// The map a tool returns: every route whose ID starts with the prefix.
export type ToolScreens<TPrefix extends string> = ScreenMap<Extract<RouteId, `${TPrefix}${string}`>>;
// The map the foundation returns: every route that is not a tool's.
export type FoundationScreens = ScreenMap<Exclude<RouteId, ToolRouteId>>;

export type AppRouterOptions = {
  context: RouterContext;
  defaultErrorComponent: ErrorRouteComponent;
  // Required: a router cannot be created without a screen for every route.
  screens: ScreenMap<RouteId>;
};

function isRouteId(router: AppRouter, id: string): id is RouteId {
  return id in router.routesById;
}

export function createAppRouter({context, defaultErrorComponent, screens}: AppRouterOptions): AppRouter {
  const router = createBareRouter(context, defaultErrorComponent);
  for (const [id, options] of Object.entries(screens)) {
    if (!isRouteId(router, id)) {
      throw new Error(`No route has the ID ${id}`);
    }
    const route: AnyRoute = router.routesById[id];
    route.update(options);
  }
  return router;
}
