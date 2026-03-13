import type {ReactNode} from 'react';
import {Suspense, useLayoutEffect} from 'react';

import type {AppConfig} from '#config';
import {useAppConfig} from '#hooks/useAppConfig';

import {PopoverPendingLayout} from './PopoverPendingLayout';

// Inject design tokens from AppConfig as CSS custom properties on the document root.
function applyDesignTokens(config: AppConfig, root: HTMLElement): void {
  const {appearance, layout} = config.popover;
  root.style.setProperty('--kodogu-surface-color', appearance.surfaceColor);
  root.style.setProperty('--kodogu-border-color', appearance.borderColor);
  root.style.setProperty('--kodogu-backdrop-filter', appearance.backdropFilter);
  root.style.setProperty('--kodogu-font-family', appearance.fontFamily);
  root.style.setProperty('--kodogu-header-height', layout.headerHeight);
  root.style.setProperty('--kodogu-card-list-padding', layout.cardListPadding);
}

// Loads all shared state required before the app can render.
// Add future shared state queries (connections, session, etc.) here.
const AppBootstrapInner = ({children}: {children: ReactNode}) => {
  const config = useAppConfig();

  useLayoutEffect(() => {
    applyDesignTokens(config, document.documentElement);
  }, [config]);

  return children;
};

// Single Suspense boundary for all app-level initialization.
export const AppBootstrap = ({children}: {children: ReactNode}) => (
  <Suspense fallback={<PopoverPendingLayout />}>
    <AppBootstrapInner>{children}</AppBootstrapInner>
  </Suspense>
);
