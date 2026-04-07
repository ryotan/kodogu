import {invoke} from '@tauri-apps/api/core';

export function resizePopover(width: number, height: number): Promise<void> {
  return invoke<void>('resize_popover', {width, height});
}
