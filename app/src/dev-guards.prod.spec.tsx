// Tests that verify dev-only components are disabled in production mode.
// Kept in a separate file so each test runs in its own browser iframe context,
// avoiding native ES module cache conflicts with static imports in other spec files.
import {afterEach, beforeEach, describe, expect, it, vi} from 'vitest';
import {page} from 'vitest/browser';
import {render} from 'vitest-browser-react';

import {QueryClient, QueryClientProvider, QueryErrorResetBoundary} from '@tanstack/react-query';

describe('dev-only components in production mode (DEV = false)', () => {
  beforeEach(() => {
    vi.resetModules();
    vi.stubEnv('DEV', false);
  });
  afterEach(() => {
    vi.unstubAllEnvs();
    vi.resetModules();
  });

  it('StackTraceInspector renders nothing', async () => {
    const {StackTraceInspector} = await import('./(dev)/StackTraceInspector');
    const {container} = await render(<StackTraceInspector />);
    expect(container.firstChild).toBeNull();
  });

  it('ErrorDisplay renders without StackTrace section', async () => {
    const {ErrorDisplay} = await import('./shared/core/errors/ErrorDisplay');
    const error = new Error('something went wrong');
    await render(<ErrorDisplay error={error} />);
    await expect.element(page.getByText('something went wrong')).toBeVisible();
    // StackTrace component renders a "Stack trace" toggle button — must be absent in production
    await expect.element(page.getByRole('button', {name: /stack trace/i})).not.toBeInTheDocument();
  });

  it('PopoverAppErrorPage renders without ComponentStack toggle', async () => {
    const {PopoverAppErrorPage} = await import('./features/menu-popover/components/PopoverAppErrorPage');
    const queryClient = new QueryClient();
    await render(
      <QueryClientProvider client={queryClient}>
        <QueryErrorResetBoundary>
          <PopoverAppErrorPage
            error={new Error('prod error')}
            info={{componentStack: '\n    at SomeComponent'}}
            reset={vi.fn()}
          />
        </QueryErrorResetBoundary>
      </QueryClientProvider>,
    );

    await expect.element(page.getByText('prod error')).toBeVisible();
    await expect.element(page.getByRole('button', {name: /try again/i})).toBeVisible();
    // ComponentStack toggle must not appear — ComponentStack is null in production
    await expect.element(page.getByRole('button', {name: /component stack/i})).not.toBeInTheDocument();
  });
});
