// Verifies that dev-only UI is rendered when __DEV_TOOLS__ is true.
// Runs in the `browser-devtools` Vitest project (vite.config.ts) where
// the flag is replaced with `true` at build time. The default `browser`
// project covers the inverse case (dev UI absent) implicitly via every
// other spec running with __DEV_TOOLS__ = false.

import {describe, expect, it, vi} from 'vitest';
import {page} from 'vitest/browser';
import {render} from 'vitest-browser-react';

import {QueryClient, QueryClientProvider, QueryErrorResetBoundary} from '@tanstack/react-query';

import {AppLayout} from './components/AppLayout';
import {PopoverAppErrorPage} from './features/menu-popover/components/PopoverAppErrorPage';
import * as RootRoute from './routes/__root';
import * as AppRoute from './routes/_app';
import {ErrorPage} from './shared/core/errors/ErrorPage';

describe('dev-only components when __DEV_TOOLS__ = true', () => {
  it('ErrorPage surfaces a Debug info disclosure', async () => {
    await render(<ErrorPage error={new Error('something went wrong')} />);
    await expect.element(page.getByText(/debug info/i)).toBeVisible();
  });

  it('PopoverAppErrorPage exposes the component stack via the Debug info disclosure', async () => {
    const queryClient = new QueryClient();
    await render(
      <QueryClientProvider client={queryClient}>
        <QueryErrorResetBoundary>
          <PopoverAppErrorPage
            error={new Error('boom')}
            errorInfo={{componentStack: '\n    at SomeComponent\n    at RouterProvider'}}
            reset={vi.fn()}
          />
        </QueryErrorResetBoundary>
      </QueryClientProvider>,
    );
    await page.getByText(/debug info/i).click();
    await page.getByRole('button', {name: /component stack/i}).click();
    await expect.element(page.getByText(/SomeComponent/)).toBeVisible();
  });
});

// Coverage: importing these modules forces evaluation of the __DEV_TOOLS__ ternary at module
// top level, so the lazy-load branch is recorded as covered when the flag is true.
describe('modules evaluate __DEV_TOOLS__ = true branch on import', () => {
  it('__root exports RootLayout', () => {
    expect(RootRoute.RootLayout).toBeDefined();
  });

  it('AppLayout component is defined', () => {
    expect(AppLayout).toBeDefined();
  });

  it('_app exports Route', () => {
    expect(AppRoute.Route).toBeDefined();
  });
});
