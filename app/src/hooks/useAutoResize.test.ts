import {afterEach, describe, expect, it, vi} from 'vitest';

import {createResizeHandler, getMaxHeight} from './useAutoResize';

describe('getMaxHeight', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('returns 85% of screen.availHeight when screen is available', () => {
    vi.stubGlobal('screen', {availHeight: 1000});
    expect(getMaxHeight()).toBe(850);
  });

  it('floors the result', () => {
    vi.stubGlobal('screen', {availHeight: 999});
    expect(getMaxHeight()).toBe(849);
  });

  it('returns fallback when screen is undefined', () => {
    vi.stubGlobal('screen', undefined);
    expect(getMaxHeight()).toBe(600);
  });
});

describe('createResizeHandler', () => {
  it('calls setSize with element dimensions on first call', () => {
    const setSize = vi.fn().mockResolvedValue(undefined);
    const handler = createResizeHandler(setSize);

    handler({offsetWidth: 320, offsetHeight: 400});

    expect(setSize).toHaveBeenCalledExactlyOnceWith(320, 400);
  });

  it('skips setSize when height has not changed', () => {
    const setSize = vi.fn().mockResolvedValue(undefined);
    const handler = createResizeHandler(setSize);

    handler({offsetWidth: 320, offsetHeight: 400});
    handler({offsetWidth: 320, offsetHeight: 400});

    expect(setSize).toHaveBeenCalledOnce();
  });

  it('calls setSize again when height changes', () => {
    const setSize = vi.fn().mockResolvedValue(undefined);
    const handler = createResizeHandler(setSize);

    handler({offsetWidth: 320, offsetHeight: 400});
    handler({offsetWidth: 320, offsetHeight: 500});

    expect(setSize).toHaveBeenCalledTimes(2);
    expect(setSize).toHaveBeenLastCalledWith(320, 500);
  });

  it('logs error when setSize rejects', async () => {
    const error = new Error('setSize failed');
    const setSize = vi.fn().mockRejectedValue(error);
    // biome-ignore lint/suspicious/noEmptyBlockStatements: suppress console.error output during test
    const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {});
    const handler = createResizeHandler(setSize);

    handler({offsetWidth: 320, offsetHeight: 400});
    // catch handler is async, so flush microtasks
    await vi.waitFor(() => {
      expect(consoleError).toHaveBeenCalledWith('Failed to resize popover window', error);
    });

    consoleError.mockRestore();
  });
});
