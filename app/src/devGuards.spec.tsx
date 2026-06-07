// Verifies that dev-only UI is omitted when __DEV_TOOLS__ is false.
// Runs in the `browser` Vitest project (vite.config.ts) where the flag is replaced with
// `false` at build time, mirroring the production bundle. The `browser-devtools` project
// covers the inverse case in `devGuards.devtools.spec.tsx`.

import {describe, expect, it, vi} from 'vitest';
import {page} from 'vitest/browser';
import {render} from 'vitest-browser-react';

import {QueryClient, QueryClientProvider, QueryErrorResetBoundary} from '@tanstack/react-query';

import {PopoverAppErrorPage} from './features/menu-popover/components/PopoverAppErrorPage';
import {ErrorDisplay} from './shared/core/errors/ErrorDisplay';

describe('dev-only components when __DEV_TOOLS__ = false', () => {
  it('ErrorDisplay does not surface a Stack trace toggle', async () => {
    await render(<ErrorDisplay error={new Error('boom')} />);
    await expect.element(page.getByRole('button', {name: /stack trace/i})).not.toBeInTheDocument();
  });

  it('PopoverAppErrorPage does not surface a Component stack toggle for componentStack info', async () => {
    const queryClient = new QueryClient();
    await render(
      <QueryClientProvider client={queryClient}>
        <QueryErrorResetBoundary>
          <PopoverAppErrorPage
            error={new Error('boom')}
            info={{componentStack: '\n    at SomeComponent\n    at RouterProvider'}}
            reset={vi.fn()}
          />
        </QueryErrorResetBoundary>
      </QueryClientProvider>,
    );
    await expect.element(page.getByRole('button', {name: /component stack/i})).not.toBeInTheDocument();
  });
});
