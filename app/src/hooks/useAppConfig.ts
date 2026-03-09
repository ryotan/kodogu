import {useSuspenseQuery} from '@tanstack/react-query';

import type {AppConfig} from '#config';
import {getConfig} from '#config';

export function useAppConfig<T = AppConfig>(select?: (config: AppConfig) => T): T {
  const {data} = useSuspenseQuery({queryKey: ['app-config'], queryFn: getConfig, select});
  return data;
}
