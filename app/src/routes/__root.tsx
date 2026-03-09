import {Suspense, useLayoutEffect} from 'react';

import {createRootRoute, Outlet} from '@tanstack/react-router';

import {TanStackTools} from '#(dev)/TanStackTools';
import type {AppConfig} from '#config';
import {useAppConfig} from '#hooks/useAppConfig';

function applyShellTokens(config: AppConfig, root: HTMLElement): void {
  const {appearance, layout} = config.popover;
  root.style.setProperty('--kodogu-surface-color', appearance.surfaceColor);
  root.style.setProperty('--kodogu-border-color', appearance.borderColor);
  root.style.setProperty('--kodogu-backdrop-filter', appearance.backdropFilter);
  root.style.setProperty('--kodogu-font-family', appearance.fontFamily);
  root.style.setProperty('--kodogu-header-height', layout.headerHeight);
  root.style.setProperty('--kodogu-card-list-padding', layout.cardListPadding);
}

const RootLayout = () => {
  const config = useAppConfig();

  useLayoutEffect(() => {
    applyShellTokens(config, document.documentElement);
  }, [config]);

  return (
    <>
      <Outlet />
      <TanStackTools />
    </>
  );
};

export const Route = createRootRoute({
  component: () => {
    return (
      <Suspense>
        <RootLayout />
      </Suspense>
    );
  },
});
