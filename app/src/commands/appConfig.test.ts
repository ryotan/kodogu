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

  it('reports an unclassified read failure without displaying its cause', async () => {
    mockInvoke.mockRejectedValueOnce(new Error('IPC error'));

    await expect(getConfig()).rejects.toMatchObject({
      name: 'ConfigurationCommandError',
      code: 'result_unavailable',
      message: 'Could not confirm the settings result. Read the current settings again.',
    });
  });

  it('reports saved values that cannot be represented by the GUI', async () => {
    mockInvoke.mockRejectedValueOnce({code: 'configuration_unrepresentable', message: 'Native diagnostic'});

    await expect(getConfig()).rejects.toMatchObject({
      name: 'ConfigurationCommandError',
      code: 'configuration_unrepresentable',
      unavailable: true,
      message: 'Saved configuration cannot be displayed safely by this interface. Your data has been kept.',
    });
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

  it.each([
    {
      code: 'configuration_rejected',
      unavailable: false,
      message: 'Configuration was not saved. Your input has been kept.',
    },
    {
      code: 'configuration_unavailable',
      unavailable: true,
      message: 'Configuration is unavailable. Restart KoDoGu before trying again.',
    },
  ])(
    'preserves the native $code classification without displaying its message',
    async ({code, unavailable, message}) => {
      mockInvoke.mockRejectedValueOnce({code, message: 'Native diagnostic'});
      await expect(setConfig(appConfigFixture)).rejects.toMatchObject({
        name: 'ConfigurationCommandError',
        code,
        unavailable,
        message,
      });
    },
  );

  it.each([
    {kind: 'unstructured', response: 'private input'},
    {kind: 'unknown-code', response: {code: 'unknown', message: 'private input'}},
    {kind: 'malformed', response: {code: 'configuration_rejected', message: {value: 'private input'}}},
  ])('keeps a write with a $kind rejection unconfirmed', async ({response}) => {
    mockInvoke.mockRejectedValueOnce(response);
    await expect(setConfig(appConfigFixture)).rejects.toMatchObject({
      name: 'ConfigurationCommandError',
      code: 'result_unavailable',
      unavailable: true,
      message: 'Could not confirm the settings result. Read the current settings again.',
    });
    expect(mockInvoke).toHaveBeenCalledExactlyOnceWith('set_config', {config: appConfigFixture});
  });
});
