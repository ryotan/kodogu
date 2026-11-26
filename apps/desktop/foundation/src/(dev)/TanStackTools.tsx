import type {ReactNode} from 'react';

import {ReactQueryDevtools} from '@tanstack/react-query-devtools';
import {TanStackRouterDevtools} from '@tanstack/react-router-devtools';

export function TanStackTools(): ReactNode {
  if (!import.meta.env.DEV) {
    return null;
  }

  return (
    <>
      <TanStackRouterDevtools />
      <ReactQueryDevtools />
    </>
  );
}
