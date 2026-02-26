import {describe, expect, it, vi} from 'vitest';
import {render, renderHook} from 'vitest-browser-react';

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => ({
    setSize: vi.fn().mockResolvedValue(undefined),
  }),
}));

vi.mock('@tauri-apps/api/dpi', () => ({
  LogicalSize: class LogicalSize {
    constructor(
      public width: number,
      public height: number,
    ) {}
  },
}));

const {useAutoResize} = await import('#hooks/useAutoResize');

// Test component that connects ref to real DOM
const TestComponent = () => {
  const {ref, maxHeight} = useAutoResize();
  return <div data-max-height={maxHeight} data-testid='container' ref={ref} style={{height: 200}} />;
};

describe('useAutoResize', () => {
  it('returns a ref and a positive maxHeight', async () => {
    const {result} = await renderHook(() => useAutoResize());

    expect(result.current.ref).toBeDefined();
    expect(result.current.maxHeight).toBeGreaterThan(0);
  });

  it('maxHeight is 85% of screen.availHeight', async () => {
    const expected = Math.floor(screen.availHeight * 0.85);
    const {result} = await renderHook(() => useAutoResize());

    expect(result.current.maxHeight).toBe(expected);
  });

  it('cleans up ResizeObserver on unmount', async () => {
    const disconnectSpy = vi.fn();
    const OriginalResizeObserver = window.ResizeObserver;
    window.ResizeObserver = class extends OriginalResizeObserver {
      disconnect() {
        disconnectSpy();
        super.disconnect();
      }
    };

    try {
      const result = await render(<TestComponent />);
      await result.unmount();
      expect(disconnectSpy).toHaveBeenCalled();
    } finally {
      window.ResizeObserver = OriginalResizeObserver;
    }
  });
});
