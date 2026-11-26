import {beforeEach, describe, expect, it, vi} from 'vitest';

import {invoke} from '@tauri-apps/api/core';
import {create} from 'mutative';

import {Milliseconds} from '#config';

import {getConfig, setConfig} from './appConfig';
import {appConfigFixture} from './appConfig.fixture';

vi.mock('@tauri-apps/api/core', () => ({invoke: vi.fn()}));

const mockInvoke = vi.mocked(invoke);

// =============================================================================
// getConfig
// =============================================================================

describe('getConfig', () => {
  beforeEach(() => mockInvoke.mockReset());

  it('invokes get_config and resolves with AppConfig', async () => {
    mockInvoke.mockResolvedValue(appConfigFixture);

    const result = await getConfig();

    expect(mockInvoke).toHaveBeenCalledWith('get_config');
    expect(result).toEqual(appConfigFixture);
  });

  it('invokes get_config and resolves with custom toggleDebounceMs', async () => {
    const custom = create(appConfigFixture, draft => {
      draft.popover.behavior.toggleDebounceMs = Milliseconds(750);
    });
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

    await setConfig(appConfigFixture);

    expect(mockInvoke).toHaveBeenCalledWith('set_config', {config: appConfigFixture});
  });

  it('invokes set_config with modified toggleDebounceMs', async () => {
    mockInvoke.mockResolvedValue(undefined);
    const config = create(appConfigFixture, draft => {
      draft.popover.behavior.toggleDebounceMs = Milliseconds(500);
    });

    await setConfig(config);

    expect(mockInvoke).toHaveBeenCalledWith('set_config', {config});
  });

  it('propagates rejection from invoke', async () => {
    mockInvoke.mockRejectedValueOnce(new Error('DB write failed'));

    await expect(setConfig(appConfigFixture)).rejects.toThrow('DB write failed');
  });
});
