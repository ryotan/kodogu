import {QueryClient, QueryClientProvider, useSuspenseQuery} from '@tanstack/react-query';
import {createMemoryHistory, createRootRoute, createRouter, RouterProvider} from '@tanstack/react-router';

import type {Meta, StoryObj} from '@storybook/react-vite';
import {expect, fn, mocked, userEvent, waitFor} from 'storybook/test';

import {reportError} from '#reportError';

import {PopoverRouteErrorPage} from './PopoverRouteErrorPage';

const meta = {
  component: PopoverRouteErrorPage,
  parameters: {layout: 'fullscreen'},
  args: {error: new Error('Route failed'), reset: fn()},
} satisfies Meta<typeof PopoverRouteErrorPage>;

export default meta;
type Story = StoryObj<typeof meta>;

export const RerunsFailedLoader: Story = {
  play: async ({mount}) => {
    const loader = fn<() => Promise<void>>()
      .mockRejectedValueOnce(new Error('loader failed'))
      .mockResolvedValue(undefined);
    const router = createRouter({
      routeTree: createRootRoute({loader, component: () => <p>loaded</p>}),
      history: createMemoryHistory({initialEntries: ['/']}),
      defaultErrorComponent: PopoverRouteErrorPage,
    });
    const queryClient = new QueryClient({defaultOptions: {queries: {retry: false}}});
    const canvas = await mount(
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>,
    );
    await userEvent.click(await canvas.findByRole('button', {name: 'Try Again'}));
    await expect(await canvas.findByText('loaded')).toBeVisible();
    await expect(loader).toHaveBeenCalledTimes(2);
  },
};

export const RefetchesFailedQuery: Story = {
  play: async ({mount}) => {
    const queryFn = fn<() => Promise<string>>()
      .mockRejectedValueOnce(new Error('query failed'))
      .mockResolvedValue('loaded');
    function Data() {
      return <p>{useSuspenseQuery({queryKey: ['data'], queryFn}).data}</p>;
    }
    const router = createRouter({
      routeTree: createRootRoute({component: Data}),
      history: createMemoryHistory({initialEntries: ['/']}),
      defaultErrorComponent: PopoverRouteErrorPage,
    });
    const queryClient = new QueryClient({defaultOptions: {queries: {retry: false}}});
    const canvas = await mount(
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>,
    );
    await userEvent.click(await canvas.findByRole('button', {name: 'Try Again'}));
    await expect(await canvas.findByText('loaded')).toBeVisible();
    await expect(queryFn).toHaveBeenCalledTimes(2);
  },
};

function createFailureStory(reason: unknown, message: string): Story {
  return {
    play: async ({mount}) => {
      const router = createRouter({
        routeTree: createRootRoute({loader: fn<() => Promise<never>>().mockRejectedValue(reason)}),
        history: createMemoryHistory({initialEntries: ['/']}),
        defaultErrorComponent: PopoverRouteErrorPage,
      });
      const canvas = await mount(<RouterProvider router={router} />);
      await expect(await canvas.findByText('Error', {exact: true})).toBeVisible();
      await expect(await canvas.findByText(message, {exact: true})).toBeVisible();
      await expect(await canvas.findByRole('button', {name: 'Try Again'})).toBeVisible();
    },
  };
}

export const StringFailure = createFailureStory('route failed', 'route failed');
export const NumberFailure = createFailureStory(42, '42');
export const ObjectFailure = createFailureStory({message: 'route failed'}, '[object Object]');

export const ReportsReloadFailure: Story = {
  beforeEach: () => {
    mocked(reportError).mockImplementation(() => undefined);
    return () => {
      mocked(reportError).mockRestore();
    };
  },
  play: async ({mount}) => {
    mocked(reportError).mockClear();
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
    const router = createRouter({
      routeTree: createRootRoute({loader: () => Promise.reject(new Error('loader failed'))}),
      history,
      defaultErrorComponent: PopoverRouteErrorPage,
    });
    const canvas = await mount(<RouterProvider router={router} />);
    const retry = await canvas.findByRole('button', {name: 'Try Again'});
    const error = new Error('History unavailable during reload');
    historyFailure = error;
    try {
      await userEvent.click(retry);
      await waitFor(async () => {
        await expect(mocked(reportError).mock.calls).toEqual([['Failed to reload the routes', error]]);
        await expect(mocked(reportError).mock.calls[0]?.[1]).toBe(error);
      });
    } finally {
      historyFailure = undefined;
    }
  },
};
