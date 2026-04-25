import {beforeEach, describe, expect, it, vi} from 'vitest';

import {invoke} from '@tauri-apps/api/core';

import type {AppConfig} from '#config';
import {Milliseconds} from '#config';

import {getConfig, setConfig} from './appConfig';
import {defaultAppConfig} from './appConfig.fixture';

vi.mock('@tauri-apps/api/core', () => ({invoke: vi.fn()}));

const mockInvoke = vi.mocked(invoke);

// =============================================================================
// getConfig
// =============================================================================

describe('getConfig', () => {
  beforeEach(() => mockInvoke.mockReset());

  it('invokes get_config and resolves with AppConfig', async () => {
    mockInvoke.mockResolvedValue(defaultAppConfig);

    const result = await getConfig();

    expect(mockInvoke).toHaveBeenCalledWith('get_config');
    expect(result).toEqual(defaultAppConfig);
  });

  it('invokes get_config and resolves with custom toggleDebounceMs', async () => {
    const custom: AppConfig = {
      ...defaultAppConfig,
      popover: {...defaultAppConfig.popover, behavior: {toggleDebounceMs: Milliseconds(750)}},
    };
    mockInvoke.mockResolvedValue(custom);

    const result = await getConfig();

    expect(result.popover.behavior.toggleDebounceMs).toBe(750);
  });

  it('propagates rejection from invoke', async () => {
    mockInvoke.mockRejectedValueOnce(new Error('IPC error'));

    await expect(getConfig()).rejects.toThrow('IPC error');
  });
});

// =============================================================================
// setConfig
// =============================================================================

describe('setConfig', () => {
  beforeEach(() => mockInvoke.mockReset());

  it('invokes set_config with config wrapped in object', async () => {
    mockInvoke.mockResolvedValue(undefined);

    await setConfig(defaultAppConfig);

    expect(mockInvoke).toHaveBeenCalledWith('set_config', {config: defaultAppConfig});
  });

  it('invokes set_config with modified toggleDebounceMs', async () => {
    mockInvoke.mockResolvedValue(undefined);
    const config: AppConfig = {
      ...defaultAppConfig,
      popover: {...defaultAppConfig.popover, behavior: {toggleDebounceMs: Milliseconds(500)}},
    };

    await setConfig(config);

    expect(mockInvoke).toHaveBeenCalledWith('set_config', {config});
  });

  it('propagates rejection from invoke', async () => {
    mockInvoke.mockRejectedValueOnce(new Error('DB write failed'));

    await expect(setConfig(defaultAppConfig)).rejects.toThrow('DB write failed');
  });
});
