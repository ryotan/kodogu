import {beforeEach, describe, expect, it, vi} from 'vitest';
import {page} from 'vitest/browser';
import {render} from 'vitest-browser-react';

import {QueryClient, QueryClientProvider, useSuspenseQuery} from '@tanstack/react-query';
import type {AnyRoute} from '@tanstack/react-router';
import {createMemoryHistory, createRootRoute, createRouter, RouterProvider} from '@tanstack/react-router';

import {PopoverRouteErrorPage} from '#components/PopoverRouteErrorPage';
import {reportError} from '#reportError';

vi.mock('#reportError', () => ({reportError: vi.fn()}));

const mockReportError = vi.mocked(reportError);

vi.mock('#commands/popover', () => ({resizePopover: vi.fn().mockResolvedValue(undefined)}));

const renderRouter = async (routeTree: AnyRoute, history = createMemoryHistory({initialEntries: ['/']})) => {
  const router = createRouter({
    routeTree,
    history,
    defaultErrorComponent: PopoverRouteErrorPage,
  });
  const queryClient = new QueryClient({defaultOptions: {queries: {retry: false}}});
  await render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
  return router;
};

describe('PopoverRouteErrorPage', () => {
  beforeEach(() => {
    mockReportError.mockClear();
  });

  it('reruns a failed loader on Try Again', async () => {
    const loader = vi
      .fn<() => Promise<void>>()
      .mockRejectedValueOnce(new Error('loader failed'))
      .mockResolvedValue(undefined);
    await renderRouter(createRootRoute({loader, component: () => <p>loaded</p>}));

    await page.getByRole('button', {name: 'Try Again'}).click();

    await expect.element(page.getByText('loaded')).toBeVisible();
    expect(loader).toHaveBeenCalledTimes(2);
  });

  it('refetches a suspense query that failed while rendering on Try Again', async () => {
    const queryFn = vi
      .fn<() => Promise<string>>()
      .mockRejectedValueOnce(new Error('query failed'))
      .mockResolvedValue('loaded');
    function Data() {
      return <p>{useSuspenseQuery({queryKey: ['data'], queryFn}).data}</p>;
    }
    await renderRouter(createRootRoute({component: Data}));

    await page.getByRole('button', {name: 'Try Again'}).click();

    await expect.element(page.getByText('loaded')).toBeVisible();
    expect(queryFn).toHaveBeenCalledTimes(2);
  });

  it('logs the error when reloading the routes fails on Try Again', async () => {
    const memoryHistory = createMemoryHistory({initialEntries: ['/']});
    let historyFailure: Error | undefined;
    const history = {
      ...memoryHistory,
      get location() {
        if (historyFailure) {
          throw historyFailure;
        }
        return memoryHistory.location;
      },
    };
    await renderRouter(createRootRoute({loader: () => Promise.reject(new Error('loader failed'))}), history);
    const reloadError = new Error('History unavailable during reload');
    historyFailure = reloadError;
    try {
      await page.getByRole('button', {name: 'Try Again'}).click();
      await vi.waitFor(() => {
        expect(mockReportError.mock.calls).toEqual([['Failed to reload the routes', reloadError]]);
        expect(mockReportError.mock.calls[0][1]).toBe(reloadError);
      });
    } finally {
      historyFailure = undefined;
    }
  });
});
