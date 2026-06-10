import {afterEach, beforeEach, describe, expect, it, vi} from 'vitest';
import {render} from 'vitest-browser-react';

const {mockInvoke} = vi.hoisted(() => ({
  mockInvoke: vi.fn().mockResolvedValue(undefined),
}));

beforeEach(() => {
  mockInvoke.mockClear();
});

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args) as unknown,
}));

import {useAutoResize} from '#hooks/useAutoResize';

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

// Wait two animation frames — the hook schedules resize via requestAnimationFrame,
// so two frames guarantees any pending observer→RAF cycle has completed.
function waitTwoFrames(): Promise<void> {
  return new Promise(resolve => {
    requestAnimationFrame(() => requestAnimationFrame(() => resolve()));
  });
}

type ResizeArgs = {width: number; height: number};

function getResizeArgs(callIndex: number): ResizeArgs {
  const args = mockInvoke.mock.calls[callIndex] as unknown[] | undefined;
  if (!args || args[0] !== 'resize_popover') {
    throw new Error(`Expected resize_popover call at index ${String(callIndex)}`);
  }
  return args[1] as ResizeArgs;
}

describe('useAutoResize', () => {
  it('calls resize_popover when content renders', async () => {
    await render(<TestComponent contentHeight={200} />);

    await vi.waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith(
        'resize_popover',
        expect.objectContaining({height: expect.any(Number) as number}) as ResizeArgs,
      );
    });
  });

  it('calls resize_popover with the correct height: content + chrome', async () => {
    await render(<TestComponent contentHeight={200} />);

    await vi.waitFor(() => expect(mockInvoke).toHaveBeenCalledTimes(1));

    // The hook computes: desiredHeight = content.clientHeight + chromeHeight
    // where chromeHeight = window.innerHeight - scrollArea.clientHeight.
    // Verify by measuring the same DOM values the hook reads.
    const content = document.querySelector('[data-testid="content"]');
    const scrollArea = content?.parentElement;
    if (!scrollArea) {
      throw new Error('Test layout: scroll area not found');
    }
    const chromeHeight = Math.max(0, window.innerHeight - scrollArea.clientHeight);

    expect(getResizeArgs(0)).toEqual({
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
    // Chrome height is constant between renders, so the difference is purely content.
    expect(getResizeArgs(1).height - getResizeArgs(0).height).toBe(200);
  });

  it('does not call resize_popover when content height is unchanged', async () => {
    const result = await render(<TestComponent contentHeight={200} />);

    await vi.waitFor(() => expect(mockInvoke).toHaveBeenCalledTimes(1));

    // Re-render with the same height — should NOT trigger a second resize.
    await result.rerender(<TestComponent contentHeight={200} />);

    // The hook uses requestAnimationFrame internally, so waiting two frames
    // guarantees any pending observer→RAF cycle has completed.
    await waitTwoFrames();
    expect(mockInvoke).toHaveBeenCalledTimes(1);
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

  it('logs error when resize_popover IPC fails', async () => {
    // mockImplementation silences the underlying console.error call so the test runner output
    // stays clean; spy.toHaveBeenCalledWith below still asserts that the log fired.
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(vi.fn());
    mockInvoke.mockRejectedValueOnce(new Error('IPC timeout'));

    try {
      await render(<TestComponent contentHeight={200} />);

      await vi.waitFor(() => expect(mockInvoke).toHaveBeenCalledWith('resize_popover', expect.anything()));
      await vi.waitFor(() => {
        expect(errorSpy).toHaveBeenCalledWith('Failed to resize popover window', expect.any(Error));
      });
    } finally {
      errorSpy.mockRestore();
    }
  });
});

// =============================================================================
// Edge cases that require mocked ResizeObserver
// =============================================================================

describe('useAutoResize — no scroll area', () => {
  let capturedCallback: ResizeObserverCallback;
  let savedResizeObserver: typeof ResizeObserver;

  beforeEach(() => {
    savedResizeObserver = window.ResizeObserver;
    window.ResizeObserver = class {
      constructor(cb: ResizeObserverCallback) {
        capturedCallback = cb;
      }
      observe = vi.fn();
      disconnect = vi.fn();
      unobserve = vi.fn();
    } as unknown as typeof ResizeObserver;

    vi.stubGlobal('requestAnimationFrame', (cb: FrameRequestCallback) => {
      cb(performance.now());
      return 0;
    });
    vi.stubGlobal('cancelAnimationFrame', vi.fn());
  });

  afterEach(() => {
    window.ResizeObserver = savedResizeObserver;
    vi.unstubAllGlobals();
  });

  it('logs error and skips resize when parentElement is null', async () => {
    // mockImplementation silences the underlying console.error call so the test runner output
    // stays clean; spy.toHaveBeenCalledWith below still asserts that the log fired.
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(vi.fn());
    mockInvoke.mockClear();

    // A component that attaches the ref to a top-level div (no intermediate scroll area).
    // When the mock ResizeObserver fires the callback with a *detached* element,
    // entry.target.parentElement is null — matching the defensive branch in the hook.
    const NoScrollAreaComponent = () => {
      const {resizeCallbackRef} = useAutoResize();
      return <div data-testid='probe' ref={resizeCallbackRef} />;
    };

    try {
      await render(<NoScrollAreaComponent />);

      // Trigger the captured callback with a detached element (parentElement === null)
      const detached = document.createElement('div');
      const entry = {
        target: detached,
        borderBoxSize: [{blockSize: 200, inlineSize: 400}],
      } as unknown as ResizeObserverEntry;
      capturedCallback([entry], {} as ResizeObserver);

      expect(errorSpy).toHaveBeenCalledWith('No scroll area was found');
      expect(mockInvoke).not.toHaveBeenCalledWith('resize_popover', expect.anything());
    } finally {
      errorSpy.mockRestore();
    }
  });
});
