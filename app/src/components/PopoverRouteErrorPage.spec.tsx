import {afterEach, beforeEach, describe, expect, it, vi} from 'vitest';
import {page} from 'vitest/browser';
import {render} from 'vitest-browser-react';

import {QueryClient, QueryClientProvider, useSuspenseQuery} from '@tanstack/react-query';
import type {AnyRoute} from '@tanstack/react-router';
import {createMemoryHistory, createRootRoute, createRouter, RouterProvider} from '@tanstack/react-router';

import {PopoverRouteErrorPage} from '#components/PopoverRouteErrorPage';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

const renderRouter = async (routeTree: AnyRoute) => {
  const router = createRouter({
    routeTree,
    history: createMemoryHistory({initialEntries: ['/']}),
    defaultErrorComponent: PopoverRouteErrorPage,
  });
  const queryClient = new QueryClient({defaultOptions: {queries: {retry: false}}});
  await render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
};

describe('PopoverRouteErrorPage', () => {
  // React and the router log the errors these tests throw on purpose.
  beforeEach(() => {
    vi.spyOn(console, 'error').mockImplementation(vi.fn());
    vi.spyOn(console, 'warn').mockImplementation(vi.fn());
  });
  afterEach(() => {
    vi.restoreAllMocks();
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
    const Data = () => <p>{useSuspenseQuery({queryKey: ['data'], queryFn}).data}</p>;
    await renderRouter(createRootRoute({component: Data}));

    await page.getByRole('button', {name: 'Try Again'}).click();

    await expect.element(page.getByText('loaded')).toBeVisible();
    expect(queryFn).toHaveBeenCalledTimes(2);
  });
});
