import {afterEach, describe, expect, it, vi} from 'vitest';

import type {InvokeArgs} from '@tauri-apps/api/core';
import {clearMocks, mockIPC} from '@tauri-apps/api/mocks';

import {resizePopover} from './popover';

afterEach(() => {
  clearMocks();
});

describe('resizePopover IPC adapter', () => {
  it('sends both dimensions to the native resize command', async () => {
    const handler = vi.fn<(command: string, payload?: InvokeArgs) => void>();
    mockIPC(handler);

    await resizePopover(640, 480);

    expect(handler.mock.calls).toEqual([['resize_popover', {width: 640, height: 480}]]);
  });

  it('propagates a native command failure', async () => {
    const error = new Error('Native resize failed');
    const handler = vi.fn<(command: string, payload?: InvokeArgs) => never>(() => {
      throw error;
    });
    mockIPC(handler);

    await expect(resizePopover(640, 480)).rejects.toBe(error);

    expect(handler.mock.calls).toEqual([['resize_popover', {width: 640, height: 480}]]);
  });
});
