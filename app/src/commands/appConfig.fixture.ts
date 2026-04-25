// Shared AppConfig fixture for tests, stories, and Storybook IPC stubs.
// Mirrors the default values that the Rust side returns from `get_config`.

import type {AppConfig} from '#config';
import {CssColor, CssFilter, CssFontFamily, CssLength, CssPadding, LogicalPixel, Milliseconds} from '#config';

export const defaultAppConfig: AppConfig = {
  popover: {
    behavior: {toggleDebounceMs: Milliseconds(300)},
    layout: {
      initialWidth: LogicalPixel(480),
      bottomMargin: LogicalPixel(8),
      trayGap: LogicalPixel(4),
      fallbackMaxHeight: LogicalPixel(600),
      minMaxHeight: LogicalPixel(200),
      headerHeight: CssLength('40px'),
      cardListPadding: CssPadding('6px 0'),
    },
    appearance: {
      surfaceColor: CssColor('rgba(28, 28, 30, 0.92)'),
      borderColor: CssColor('rgba(255, 255, 255, 0.08)'),
      backdropFilter: CssFilter('blur(24px) saturate(180%)'),
      fontFamily: CssFontFamily("-apple-system, BlinkMacSystemFont, 'Helvetica Neue', sans-serif"),
    },
  },
};

// Convenience builder for tests that only tweak appearance values.
export function createAppConfig(overrides?: Partial<AppConfig['popover']['appearance']>): AppConfig {
  return {
    popover: {
      ...defaultAppConfig.popover,
      appearance: {...defaultAppConfig.popover.appearance, ...overrides},
    },
  };
}
