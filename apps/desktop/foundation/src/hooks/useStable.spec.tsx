import {describe, expect, it, vi} from 'vitest';
import {renderHook} from 'vitest-browser-react';

import {useStable} from '#hooks/useStable';

describe('useStable', () => {
  it('keeps the first instance across re-renders', async () => {
    const initializer = vi.fn(() => ({}));
    const {result, rerender} = await renderHook(() => useStable(initializer));
    const first = result.current;

    await rerender();

    expect(result.current).toBe(first);
    expect(initializer).toHaveBeenCalledOnce();
  });

  it('creates a new instance for a new mount', async () => {
    const initializer = () => ({});
    const {result: a} = await renderHook(() => useStable(initializer));
    const {result: b} = await renderHook(() => useStable(initializer));

    expect(a.current).not.toBe(b.current);
  });
});
