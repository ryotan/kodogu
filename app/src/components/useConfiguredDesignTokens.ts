import {useLayoutEffect} from 'react';

import type {AppConfig} from '#config';
import {useAppConfig} from '#hooks/useAppConfig';

function applyDesignTokens(config: AppConfig, root: HTMLElement): void {
  const {appearance, layout} = config.popover;
  root.style.setProperty('--kodogu-surface-color', appearance.surfaceColor);
  root.style.setProperty('--kodogu-border-color', appearance.borderColor);
  root.style.setProperty('--kodogu-backdrop-filter', appearance.backdropFilter);
  root.style.setProperty('--kodogu-font-family', appearance.fontFamily);
  root.style.setProperty('--kodogu-header-height', layout.headerHeight);
  root.style.setProperty('--kodogu-card-list-padding', layout.cardListPadding);
}

export const useConfiguredDesignTokens = () => {
  const config = useAppConfig();
  useLayoutEffect(() => {
    applyDesignTokens(config, document.documentElement);
  }, [config]);
};
