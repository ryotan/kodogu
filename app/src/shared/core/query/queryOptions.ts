import {queryOptions as _queryOptions} from '@tanstack/react-query';
import type {QueryFunction, QueryKey, UndefinedInitialDataOptions} from '@tanstack/react-query';

import {RootError} from '#shared/core/errors/RootError';

export class QueryCallSiteError extends RootError {
  readonly queryKey: QueryKey;

  constructor(queryKey: QueryKey) {
    super(`Query called with key: ${JSON.stringify(queryKey)}`);
    this.queryKey = queryKey;
  }
}

export function queryOptions<T, K extends QueryKey = QueryKey>(
  options: UndefinedInitialDataOptions<T, Error, T, K>,
) {
  const {queryFn, ...rest} = options;
  // skipToken is a truthy symbol, so `!queryFn` would not catch it — use typeof
  if (typeof queryFn !== 'function') return _queryOptions(options);

  return _queryOptions({
    ...rest,
    queryFn: (ctx) => {
      const callSite = new QueryCallSiteError(ctx.queryKey);
      return Promise.resolve()
        .then(() => (queryFn as QueryFunction<T, K>)(ctx))
        .catch((e: unknown) => {
          // Attach call site only when no cause exists — preserve existing cause chains
          if (e instanceof Error && !e.cause) e.cause = callSite;
          throw e;
        });
    },
  });
}
