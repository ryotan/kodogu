import type {AppConfig} from '#config';
import {CssColor, CssFilter, CssFontFamily, CssLength, CssPadding, LogicalPixel, Milliseconds} from '#config';

// Stands in for what `get_config` returns.
export const appConfigFixture: AppConfig = {
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
