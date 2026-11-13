import {invoke} from '@tauri-apps/api/core';

import type {AppConfig} from '#config';

import {ConfigurationCommandError} from './ConfigurationCommandError';

export function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>('get_config').catch((error: unknown) => {
    throw ConfigurationCommandError.from(error);
  });
}

export function setConfig(config: AppConfig): Promise<void> {
  return invoke<void>('set_config', {config}).catch((error: unknown) => {
    throw ConfigurationCommandError.from(error);
  });
}
