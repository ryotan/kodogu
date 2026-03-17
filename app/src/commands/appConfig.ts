import {invoke} from '@tauri-apps/api/core';

import type {AppConfig} from '#config';

export function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>('get_config');
}

export function setConfig(config: AppConfig): Promise<void> {
  return invoke<void>('set_config', {config});
}
