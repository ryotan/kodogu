import type {QueryClient} from '@tanstack/react-query';

import type {AppConfig} from '#config';
import {appConfigQuery} from '#hooks/useAppConfig';

// What the root route runs before the first screen; the package that creates the router passes it in.
// `staleTime: 'static'` is load-bearing: it is the only staleTime `isStaleByTime()` short-circuits on, so the
// loader fetches solely while the cache is empty. A numeric one, `Infinity` included, yields to `isInvalidated`.
export function loadAppConfig(queryClient: QueryClient): Promise<AppConfig> {
  return queryClient.query({...appConfigQuery, staleTime: 'static'});
}
