import {useSuspenseQuery} from '@tanstack/react-query';

import type {AppConfig} from '#config';
import {getConfig} from '#config';
import {QueryKeys} from '#QueryKeys';

export function useAppConfig<T = AppConfig>(select?: (config: AppConfig) => T): T {
  const {data} = useSuspenseQuery({queryKey: QueryKeys.appConfig(), queryFn: getConfig, select});
  return data;
}
