import type {QueryFunction, QueryKey, UndefinedInitialDataOptions} from '@tanstack/react-query';
import {queryOptions as _queryOptions} from '@tanstack/react-query';

import {RootError} from '#shared/core/errors/RootError';

export class QueryCallSiteError extends RootError {
  readonly queryKey: QueryKey;

  constructor(queryKey: QueryKey) {
    super(`Query called with key: ${JSON.stringify(queryKey)}`);
    this.queryKey = queryKey;
  }
}

// When queryFn is a concrete function, preserve that in the return type so
// callers like useSuspenseQuery (which disallows skipToken) can accept the result.
// ReturnType<typeof _queryOptions<...>> preserves TanStack's DataTag for getQueryData() inference.
export function queryOptions<TQueryFnData, TData = TQueryFnData, K extends QueryKey = QueryKey>(
  options: UndefinedInitialDataOptions<TQueryFnData, Error, TData, K> & {queryFn: QueryFunction<TQueryFnData, K>},
): ReturnType<typeof _queryOptions<TQueryFnData, Error, TData, K>> & {queryFn: QueryFunction<TQueryFnData, K>};
export function queryOptions<TQueryFnData, TData = TQueryFnData, K extends QueryKey = QueryKey>(
  options: UndefinedInitialDataOptions<TQueryFnData, Error, TData, K>,
): ReturnType<typeof _queryOptions<TQueryFnData, Error, TData, K>>;
export function queryOptions<TQueryFnData, TData = TQueryFnData, K extends QueryKey = QueryKey>(
  options: UndefinedInitialDataOptions<TQueryFnData, Error, TData, K>,
): ReturnType<typeof _queryOptions<TQueryFnData, Error, TData, K>> {
  const {queryFn, ...rest} = options;
  // Ternary instead of early-return — produces a single _queryOptions call so
  // TypeScript infers one return type rather than a union of two branches.
  // In the true branch, TypeScript narrows queryFn to QueryFunction<TQueryFnData, K>,
  // so queryFn(ctx) requires no cast.
  // skipToken is a truthy symbol (typeof === 'symbol'), not a function, so it is
  // always handled by the false branch (passed through unchanged).
  const wrappedQueryFn: typeof queryFn =
    typeof queryFn === 'function'
      ? ctx => {
          const callSite = new QueryCallSiteError(ctx.queryKey);
          return Promise.resolve()
            .then(() => queryFn(ctx))
            .catch((e: unknown) => {
              // Attach call site only when no cause exists — preserve existing cause chains
              if (e instanceof Error && !e.cause) {
                try {
                  e.cause = callSite;
                } catch {
                  // e is frozen — cannot mutate; skip enrichment
                }
              }
              throw e;
            });
        }
      : queryFn;
  return _queryOptions({...rest, queryFn: wrappedQueryFn});
}
