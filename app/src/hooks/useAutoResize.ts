import {useCallback} from 'react';

import {resizePopover} from '#commands/popover';
import {reportError} from '#reportError';

type WindowSizeMeasurement = {
  windowInnerWidth: number;
  windowInnerHeight: number;
  scrollAreaHeight: number;
  chromeHeight: number;
  contentHeight: number;
  desiredHeight: number;
};

function measureDesiredWindowSize(entry: ResizeObserverEntry): WindowSizeMeasurement | null {
  // Ensure scroll area exists
  const scrollArea = entry.target.parentElement;
  if (!scrollArea) {
    reportError('No scroll area was found');
    return null;
  }

  // Calculate chrome height and desired height.
  // borderBoxSize[0].blockSize is the type-safe equivalent of offsetHeight from ResizeObserver —
  // includes border, padding, and content without requiring HTMLElement type narrowing.
  const windowInnerWidth = window.innerWidth;
  const windowInnerHeight = window.innerHeight;
  const scrollAreaHeight = scrollArea.clientHeight;
  const chromeHeight = Math.max(0, windowInnerHeight - scrollAreaHeight);
  const contentHeight = entry.borderBoxSize[0]?.blockSize ?? 0;
  const desiredHeight = contentHeight + chromeHeight;

  return {windowInnerWidth, windowInnerHeight, scrollAreaHeight, chromeHeight, contentHeight, desiredHeight};
}

function invokeResizePopoverCommand(measurement: WindowSizeMeasurement) {
  resizePopover(measurement.windowInnerWidth, measurement.desiredHeight).catch((e: unknown) => {
    reportError('Failed to resize popover window', e);
  });
}

// Resizes the Tauri window to match the observed element's natural height.
//
// Place the ref on the content wrapper INSIDE the scroll area.
// The hook computes the "chrome" height (header, footer, borders — everything
// except the scroll area) as: window.innerHeight - scrollArea.clientHeight.
// Desired window height = content.offsetHeight + chrome.
//
// The Rust side holds the max height constraint and caps the resize request
// via the `resize_popover` command. No OS-level set_max_size is used.
//
// Uses React 19 cleanup callback ref: returning a cleanup function from the
// ref callback lets all state live as closure-local variables, eliminating useRef.
export function useAutoResize(): {resizeCallbackRef: (content: HTMLElement | null) => void} {
  const resizeCallbackRef = useCallback((content: HTMLElement | null) => {
    if (!content) {
      return;
    }

    let previousHeight = 0;
    let frameRequestId = 0;

    const observer = new ResizeObserver(entries => {
      // ResizeObserver may batch multiple entries; use the last one (the most recent state).
      const entry = entries[entries.length - 1];
      cancelAnimationFrame(frameRequestId);
      frameRequestId = requestAnimationFrame(() => {
        const measurement = measureDesiredWindowSize(entry);
        if (!measurement) {
          return;
        }

        // Only update if the content height has changed
        if (measurement.contentHeight === previousHeight) {
          return;
        }
        previousHeight = measurement.contentHeight;

        invokeResizePopoverCommand(measurement);
      });
    });
    observer.observe(content);

    // React 19 cleanup callback ref — called when an element detaches or ref changes.
    return () => {
      observer.disconnect();
      cancelAnimationFrame(frameRequestId);
    };
  }, []);

  return {resizeCallbackRef};
}
