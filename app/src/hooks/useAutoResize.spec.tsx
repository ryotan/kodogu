import {beforeEach, describe, expect, it, vi} from 'vitest';
import {render} from 'vitest-browser-react';

const mockInvoke = vi.fn().mockResolvedValue(undefined);

beforeEach(() => {
  mockInvoke.mockClear();
});

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

const {useAutoResize} = await import('#hooks/useAutoResize');

// Simulates the popover layout: outer (viewport) > scroll area > content (ref).
const TestComponent = ({contentHeight}: {contentHeight: number}) => {
  const {ref} = useAutoResize();
  return (
    <div style={{height: 500, display: 'flex', flexDirection: 'column'}}>
      <div style={{height: 40, flexShrink: 0}}>header</div>
      <div style={{flex: 1, overflow: 'auto', minHeight: 0}}>
        <div ref={ref} style={{height: contentHeight}} data-testid='content' />
      </div>
      <div style={{height: 40, flexShrink: 0}}>footer</div>
    </div>
  );
};

describe('useAutoResize', () => {
  it('calls resize_popover when content renders', async () => {
    await render(<TestComponent contentHeight={200} />);

    await vi.waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('resize_popover', expect.objectContaining({height: expect.any(Number)}));
    });
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
