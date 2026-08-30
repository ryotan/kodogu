// Verifies that dev-only UI is omitted when __DEV_TOOLS__ is false.
// Runs in the `browser` Vitest project (vite.config.ts) where the flag is replaced with
// `false` at build time, mirroring the production bundle. The `browser-devtools` project
// covers the inverse case in `devGuards.devtools.spec.tsx`.

import {describe, expect, it, vi} from 'vitest';
import {page} from 'vitest/browser';
import {render} from 'vitest-browser-react';

import {QueryClient, QueryClientProvider, QueryErrorResetBoundary} from '@tanstack/react-query';

import {PopoverAppErrorPage} from './features/menu-popover/components/PopoverAppErrorPage';
import {ErrorPage} from './shared/core/errors/ErrorPage';

describe('dev-only components when __DEV_TOOLS__ = false', () => {
  it('ErrorPage does not surface a Debug info disclosure', async () => {
    await render(<ErrorPage error={new Error('boom')} />);
    await expect.element(page.getByText(/debug info/i)).not.toBeInTheDocument();
  });

  it('PopoverAppErrorPage does not surface a Debug info disclosure even when componentStack is supplied', async () => {
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
    await expect.element(page.getByText(/debug info/i)).not.toBeInTheDocument();
  });
});
