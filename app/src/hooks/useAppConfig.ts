import type {QueryKey, QueryKeyWithDataTag, UnusedSkipTokenOptions} from '@tanstack/react-query';
import {queryOptions, useSuspenseQuery} from '@tanstack/react-query';

import {getConfig} from '#commands/appConfig';
import type {AppConfig} from '#config';

export type AppConfigQueryOptions<T = AppConfig> = Omit<
  Parameters<typeof queryOptions<AppConfig, Error, T>>[0],
  'queryKey' | 'queryFn' | 'staleTime' | 'gcTime'
>;

type AppConfigQuery<T> = UnusedSkipTokenOptions<AppConfig, Error, T> & QueryKeyWithDataTag<QueryKey, AppConfig>;

export function buildAppConfigQuery<T = AppConfig>(opts?: AppConfigQueryOptions<T>): AppConfigQuery<T> {
  return queryOptions({
    queryKey: ['app-config'] as const,
    queryFn: getConfig,
    staleTime: Number.POSITIVE_INFINITY,
    gcTime: Number.POSITIVE_INFINITY,
    ...opts,
  });
}

export const appConfigQuery = buildAppConfigQuery();

export function useAppConfig<T = AppConfig>(select?: (config: AppConfig) => T): T {
  const {data} = useSuspenseQuery(buildAppConfigQuery({select}));
  return data;
}
