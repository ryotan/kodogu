import type {RefObject} from 'react';
import {useEffect, useRef} from 'react';

import {LogicalSize} from '@tauri-apps/api/dpi';
import {getCurrentWindow} from '@tauri-apps/api/window';

const MAX_HEIGHT_RATIO = 0.85;
const FALLBACK_MAX_HEIGHT = 600;

// Cap at 85% of display height.
// screen.availHeight is primary-monitor only (Technical Debt #5).
export function getMaxHeight(): number {
  if (typeof screen !== 'undefined') {
    return Math.floor(screen.availHeight * MAX_HEIGHT_RATIO);
  }
  return FALLBACK_MAX_HEIGHT;
}

// Creates a callback that calls setSize on height changes.
// Includes deduplication to skip consecutive calls with the same height.
export function createResizeHandler(setSize: (width: number, height: number) => Promise<void>) {
  let lastHeight = 0;
  return (el: {offsetWidth: number; offsetHeight: number}) => {
    const height = el.offsetHeight;
    if (height === lastHeight) {
      return;
    }
    lastHeight = height;
    // TODO: Replace console.error with structured logging when frontend logging is set up.
    //       console.error is invisible in production Tauri builds (no DevTools).
    setSize(el.offsetWidth, height).catch((e: unknown) => {
      console.error('Failed to resize popover window', e);
    });
  };
}

// Hook that resizes the Tauri window to match content height.
// Watches DOM height via ResizeObserver and calls setSize on changes.
export function useAutoResize(): {ref: RefObject<HTMLDivElement | null>; maxHeight: number} {
  const ref = useRef<HTMLDivElement>(null);
  const maxHeight = getMaxHeight();

  useEffect(() => {
    const el = ref.current;
    if (!el) {
      return;
    }

    const onResize = createResizeHandler((width, height) => getCurrentWindow().setSize(new LogicalSize(width, height)));
    const observer = new ResizeObserver(() => onResize(el));
    observer.observe(el);

    return () => observer.disconnect();
  }, []);

  return {ref, maxHeight};
}
