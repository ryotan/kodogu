import {beforeEach, describe, expect, it, vi} from 'vitest';
import {render} from 'vitest-browser-react';

const mockInvoke = vi.fn().mockResolvedValue(undefined);

beforeEach(() => {
  mockInvoke.mockClear();
});

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args) as unknown,
}));

const {useAutoResize} = await import('#hooks/useAutoResize');

// Simulates the popover layout: outer (viewport) > scroll area > content (ref).
const TestComponent = ({contentHeight}: {contentHeight: number}) => {
  const {resizeCallbackRef} = useAutoResize();
  return (
    <div style={{height: 500, display: 'flex', flexDirection: 'column'}}>
      <div style={{height: 40, flexShrink: 0}}>header</div>
      <div style={{flex: 1, overflow: 'auto', minHeight: 0}}>
        <div data-testid='content' ref={resizeCallbackRef} style={{height: contentHeight}} />
      </div>
      <div style={{height: 40, flexShrink: 0}}>footer</div>
    </div>
  );
};

describe('useAutoResize', () => {
  it('calls resize_popover when content renders', async () => {
    await render(<TestComponent contentHeight={200} />);

    await vi.waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith(
        'resize_popover',
        expect.objectContaining({height: expect.any(Number) as unknown}),
      );
    });
  });

  it('calls resize_popover with the correct height: content + chrome', async () => {
    await render(<TestComponent contentHeight={200} />);

    await vi.waitFor(() => expect(mockInvoke).toHaveBeenCalledTimes(1));

    // Compute expected height from actual DOM layout.
    // chrome = everything outside the scroll area (header + footer + borders).
    const content = document.querySelector('[data-testid="content"]');
    const scrollArea = content?.parentElement;
    if (scrollArea == null) {
      throw new Error('no scroll area');
    }
    const chromeHeight = Math.max(0, window.innerHeight - scrollArea.clientHeight);

    expect(mockInvoke).toHaveBeenCalledWith('resize_popover', {
      width: window.innerWidth,
      height: 200 + chromeHeight,
    });
  });

  it('calls resize_popover again when content height changes', async () => {
    const result = await render(<TestComponent contentHeight={200} />);

    await vi.waitFor(() => expect(mockInvoke).toHaveBeenCalledTimes(1));

    await result.rerender(<TestComponent contentHeight={400} />);

    await vi.waitFor(() => expect(mockInvoke).toHaveBeenCalledTimes(2));

    // The height delta must match the content height delta exactly.
    // chrome height is constant between renders, so the difference is purely content.
    const firstHeight = (mockInvoke.mock.calls[0][1] as {height: number}).height;
    const secondHeight = (mockInvoke.mock.calls[1][1] as {height: number}).height;
    expect(secondHeight - firstHeight).toBe(200);
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
      const result = await render(<TestComponent contentHeight={200} />);
      await result.unmount();
      expect(disconnectSpy).toHaveBeenCalled();
    } finally {
      window.ResizeObserver = OriginalResizeObserver;
    }
  });
});
