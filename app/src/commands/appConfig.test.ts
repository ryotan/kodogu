import {beforeEach, describe, expect, it, vi} from 'vitest';

import {invoke} from '@tauri-apps/api/core';

import type {AppConfig} from '#config';
import {CssColor, CssFilter, CssFontFamily, CssLength, CssPadding, LogicalPixel, Milliseconds} from '#config';

import {getConfig, setConfig} from './appConfig';

vi.mock('@tauri-apps/api/core', () => ({invoke: vi.fn()}));

const mockInvoke = vi.mocked(invoke);

const defaultConfig: AppConfig = {
  popover: {
    behavior: {toggleDebounceMs: Milliseconds(300)},
    layout: {
      initialWidth: LogicalPixel(480),
      bottomMargin: LogicalPixel(8),
      trayGap: LogicalPixel(4),
      fallbackMaxHeight: LogicalPixel(600),
      minMaxHeight: LogicalPixel(200),
      headerHeight: CssLength('40px'),
      cardListPadding: CssPadding('6px 0'),
    },
    appearance: {
      surfaceColor: CssColor('rgba(28, 28, 30, 0.92)'),
      borderColor: CssColor('rgba(255, 255, 255, 0.08)'),
      backdropFilter: CssFilter('blur(24px) saturate(180%)'),
      fontFamily: CssFontFamily("-apple-system, BlinkMacSystemFont, 'Helvetica Neue', sans-serif"),
    },
  },
};

// =============================================================================
// getConfig
// =============================================================================

describe('getConfig', () => {
  beforeEach(() => mockInvoke.mockReset());

  it('invokes get_config and resolves with AppConfig', async () => {
    mockInvoke.mockResolvedValue(defaultConfig);

    const result = await getConfig();

    expect(mockInvoke).toHaveBeenCalledWith('get_config');
    expect(result).toEqual(defaultConfig);
  });

  it('invokes get_config and resolves with custom toggleDebounceMs', async () => {
    const custom: AppConfig = {
      ...defaultConfig,
      popover: {...defaultConfig.popover, behavior: {toggleDebounceMs: Milliseconds(750)}},
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

    await setConfig(defaultConfig);

    expect(mockInvoke).toHaveBeenCalledWith('set_config', {config: defaultConfig});
  });

  it('invokes set_config with modified toggleDebounceMs', async () => {
    mockInvoke.mockResolvedValue(undefined);
    const config: AppConfig = {
      ...defaultConfig,
      popover: {...defaultConfig.popover, behavior: {toggleDebounceMs: Milliseconds(500)}},
    };

    await setConfig(config);

    expect(mockInvoke).toHaveBeenCalledWith('set_config', {config});
  });

  it('propagates rejection from invoke', async () => {
    mockInvoke.mockRejectedValueOnce(new Error('DB write failed'));

    await expect(setConfig(defaultConfig)).rejects.toThrow('DB write failed');
  });
});
