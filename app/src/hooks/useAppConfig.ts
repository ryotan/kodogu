import {useSuspenseQuery} from '@tanstack/react-query';

import {getConfig} from '#commands/appConfig';
import type {AppConfig} from '#config';
import {queryOptions} from '#shared/core/query/queryOptions';

export type AppConfigQueryOptions<T = AppConfig> = Omit<
  Parameters<typeof queryOptions<AppConfig, T>>[0],
  'queryKey' | 'queryFn' | 'staleTime' | 'gcTime'
>;

export const buildAppConfigQuery = <T = AppConfig>(opts?: AppConfigQueryOptions<T>) =>
  queryOptions({
    queryKey: ['app-config'] as const,
    queryFn: getConfig,
    staleTime: Number.POSITIVE_INFINITY,
    gcTime: Number.POSITIVE_INFINITY,
    ...opts,
  });

export const appConfigQuery = buildAppConfigQuery();

export function useAppConfig<T = AppConfig>(select?: (config: AppConfig) => T): T {
  const {data} = useSuspenseQuery(buildAppConfigQuery({select}));
  return data;
}
