import {useCallback, useRef} from 'react';

import {invoke} from '@tauri-apps/api/core';

type WindowSizeMeasurement = {
  windowInnerWidth: number;
  windowInnerHeight: number;
  scrollAreaHeight: number;
  chromeHeight: number;
  contentHeight: number;
  desiredHeight: number;
};

function measureDesiredWindowSize(content: Element): WindowSizeMeasurement | null {
  // Ensure scroll area exists
  const scrollArea = content.parentElement;
  if (!scrollArea) {
    console.error('No scroll area was found');
    return null;
  }

  // Calculate chrome height and desired height
  const windowInnerWidth = window.innerWidth;
  const windowInnerHeight = window.innerHeight;
  const scrollAreaHeight = scrollArea.clientHeight;
  const chromeHeight = Math.max(0, windowInnerHeight - scrollAreaHeight);
  const contentHeight = content.clientHeight;
  const desiredHeight = contentHeight + chromeHeight;

  return {windowInnerWidth, windowInnerHeight, scrollAreaHeight, chromeHeight, contentHeight, desiredHeight};
}

function invokeResizePopoverCommand(measurement: WindowSizeMeasurement, timestamp: number) {
  console.trace(`invoking resize_popover at ${timestamp}`, measurement);

  invoke('resize_popover', {
    width: measurement.windowInnerWidth,
    height: measurement.desiredHeight,
  }).catch((e: unknown) => {
    console.error('Failed to resize popover window', e);
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
export function useAutoResize(): {resizeCallbackRef: (content: HTMLElement | null) => void} {
  const attached = useRef<ResizeObserver | null>(null);
  const previousHeight = useRef(0);
  const frameRequestId = useRef(0);

  const observer = useCallback((content: HTMLElement | null) => {
    // Clean up previous observer
    if (attached.current) {
      attached.current.disconnect();
      attached.current = null;
    }
    // Cancel any pending frame request when content changes
    cancelAnimationFrame(frameRequestId.current);

    if (!content) {
      return;
    }

    attached.current = new ResizeObserver(entries => {
      if (entries.length !== 1) {
        console.error('Expected exactly one ResizeObserver entry');
        return;
      }

      const content = entries[0].target;
      cancelAnimationFrame(frameRequestId.current);
      frameRequestId.current = requestAnimationFrame(timestamp => {
        const measurement = measureDesiredWindowSize(content);
        // Only update if precondition is met
        if (!measurement) {
          return;
        }

        // Only update if the content height has changed
        if (measurement.contentHeight === previousHeight.current) {
          return;
        }
        previousHeight.current = measurement.contentHeight;

        invokeResizePopoverCommand(measurement, timestamp);
      });
    });
    attached.current.observe(content);
  }, []);

  return {resizeCallbackRef: observer};
}
