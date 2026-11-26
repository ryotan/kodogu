import type {ReactElement} from 'react';

import {Outlet} from '@tanstack/react-router';

import {TanStackTools} from '#(dev)/TanStackTools';
import {useConfiguredDesignTokens} from '#hooks/useConfiguredDesignTokens';

export function RootLayout(): ReactElement {
  useConfiguredDesignTokens();
  return (
    <>
      <Outlet />
      <TanStackTools />
    </>
  );
}
