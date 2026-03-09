import type {RefObject} from 'react';
import {useEffect, useRef} from 'react';

import {invoke} from '@tauri-apps/api/core';

// Resizes the Tauri window to match the observed element's natural height.
//
// Place the ref on the content wrapper INSIDE the scroll area.
// The hook computes the "chrome" height (header, footer, borders — everything
// except the scroll area) as: window.innerHeight - scrollArea.clientHeight.
// Desired window height = content.offsetHeight + chrome.
//
// The Rust side holds the max height constraint and caps the resize request
// via the `resize_popover` command. No OS-level set_max_size is used.
export function useAutoResize(): {ref: RefObject<HTMLDivElement | null>} {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const content = ref.current;
    if (!content) {
      return;
    }

    let lastHeight = 0;
    let rafId = 0;

    const observer = new ResizeObserver(() => {
      cancelAnimationFrame(rafId);
      rafId = requestAnimationFrame(() => {
        const contentHeight = content.offsetHeight;
        if (contentHeight === lastHeight) {
          return;
        }
        lastHeight = contentHeight;

        const scrollArea = content.parentElement;
        if (!scrollArea) {
          return;
        }

        const chromeHeight = Math.max(0, window.innerHeight - scrollArea.clientHeight);
        const desiredHeight = contentHeight + chromeHeight;

        console.debug('[useAutoResize]', {
          contentHeight,
          innerHeight: window.innerHeight,
          scrollClientH: scrollArea.clientHeight,
          chromeHeight,
          desiredHeight,
        });

        invoke('resize_popover', {width: window.innerWidth, height: desiredHeight}).catch((e: unknown) => {
          console.error('Failed to resize popover window', e);
        });
      });
    });

    observer.observe(content);
    return () => {
      cancelAnimationFrame(rafId);
      observer.disconnect();
    };
  }, []);

  return {ref};
}
