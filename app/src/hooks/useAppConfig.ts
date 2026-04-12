import {useSuspenseQuery} from '@tanstack/react-query';

import {queryOptions} from '#shared/core/query/queryOptions';

import {getConfig} from '#commands/appConfig';
import type {AppConfig} from '#config';

// TODO: Replace `Error` with Rust-side IPC error type once IPC error handling is improved. (KDG-27)
export type AppConfigQueryOptions<T = AppConfig> = Omit<
  Parameters<typeof queryOptions<AppConfig, Error, T>>[0],
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
