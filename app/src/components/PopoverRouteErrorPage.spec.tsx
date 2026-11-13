import {beforeEach, describe, expect, it, vi} from 'vitest';
import {page} from 'vitest/browser';
import {render} from 'vitest-browser-react';

import type {ReactElement} from 'react';
import {useEffect} from 'react';

import {QueryClient, QueryClientProvider, QueryErrorResetBoundary, useSuspenseQuery} from '@tanstack/react-query';
import type {AnyRoute} from '@tanstack/react-router';
import {createMemoryHistory, createRootRoute, createRoute, createRouter, RouterProvider} from '@tanstack/react-router';

import {getConfig, setConfig} from '#commands/appConfig';
import {appConfigFixture} from '#commands/appConfig.fixture';
import {ConfigurationCommandError} from '#commands/ConfigurationCommandError';
import {PopoverRouteErrorPage} from '#components/PopoverRouteErrorPage';
import type {AppConfig} from '#config';
import {Milliseconds} from '#config';
import {AppConfigCommandsProvider} from '#hooks/AppConfigCommandsProvider';
import {useAppConfig} from '#hooks/useAppConfig';
import {useAppConfigCommands} from '#hooks/useAppConfigCommands';
import {reportError} from '#reportError';
import {Route} from '#routes/__root';

vi.mock('#commands/appConfig', () => ({getConfig: vi.fn(), setConfig: vi.fn()}));
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

  it.each([
    {kind: 'string', reason: 'route failed', message: 'route failed'},
    {kind: 'number', reason: 42, message: '42'},
    {kind: 'object', reason: {message: 'route failed'}, message: '[object Object]'},
  ])('displays a $kind route failure without losing the error page', async ({reason, message}) => {
    const loader = vi.fn<() => Promise<never>>().mockRejectedValue(reason);
    await renderRouter(createRootRoute({loader}));

    await expect.element(page.getByText('Error', {exact: true})).toBeVisible();
    await expect.element(page.getByText(message, {exact: true})).toBeVisible();
    await expect.element(page.getByRole('button', {name: 'Try Again'})).toBeVisible();
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

function configurationWithDebounce(value: number): AppConfig {
  return {
    ...appConfigFixture,
    popover: {...appConfigFixture.popover, behavior: {toggleDebounceMs: Milliseconds(value)}},
  };
}

function renderSettingsRouter(component: () => ReactElement) {
  const queryClient = new QueryClient({defaultOptions: {queries: {retry: false}, mutations: {retry: false}}});
  const child = createRoute({getParentRoute: () => Route, path: '/', component});
  const router = createRouter({
    routeTree: Route.addChildren([child]),
    context: {queryClient},
    history: createMemoryHistory({initialEntries: ['/']}),
    defaultErrorComponent: PopoverRouteErrorPage,
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <AppConfigCommandsProvider>
        <QueryErrorResetBoundary>
          <RouterProvider router={router} />
        </QueryErrorResetBoundary>
      </AppConfigCommandsProvider>
    </QueryClientProvider>,
  );
}

it('stops displaying unavailable settings and retries their native read without resending input', async () => {
  const read = vi.mocked(getConfig);
  const write = vi.mocked(setConfig);
  const failure = new ConfigurationCommandError('configuration_unavailable');
  let unavailable = false;
  read.mockImplementation(() => (unavailable ? Promise.reject(failure) : Promise.resolve(appConfigFixture)));
  write.mockImplementation(() => {
    unavailable = true;
    return Promise.reject(failure);
  });
  function Settings() {
    const config = useAppConfig();
    const {update} = useAppConfigCommands();
    return (
      <>
        <p>Confirmed: {config.popover.behavior.toggleDebounceMs}</p>
        <button
          onClick={() =>
            update(draft => {
              draft.popover.behavior.toggleDebounceMs = Milliseconds(999);
            })
          }
          type='button'
        >
          Save settings
        </button>
      </>
    );
  }
  const view = await renderSettingsRouter(Settings);
  try {
    await expect.element(page.getByText('Confirmed: 300')).toBeVisible();
    await page.getByRole('button', {name: 'Save settings'}).click();
    await expect
      .element(page.getByText('Configuration is unavailable. Restart KoDoGu before trying again.'))
      .toBeVisible();
    await expect.element(page.getByText('Confirmed: 300')).not.toBeInTheDocument();
    const readsBefore = read.mock.calls.length;
    await page.getByRole('button', {name: 'Try Again'}).click();
    await vi.waitFor(() => expect(read.mock.calls.length).toBeGreaterThan(readsBefore));
    expect(write.mock.calls).toEqual([[configurationWithDebounce(999)]]);
    await expect
      .element(page.getByText('Configuration is unavailable. Restart KoDoGu before trying again.'))
      .toBeVisible();
  } finally {
    await view.unmount();
    read.mockReset();
    write.mockReset();
  }
});

it.each([
  {valueRead: 'the previous value', debounce: 300},
  {valueRead: 'the submitted value', debounce: 999},
])('retains input and the lost write result after a fresh read returns $valueRead', async ({debounce}) => {
  const read = vi.mocked(getConfig);
  const write = vi.mocked(setConfig);
  const failure = new ConfigurationCommandError('result_unavailable');
  const observeMount = vi.fn<(phase: 'mount' | 'unmount') => void>();
  let replyLost = false;
  read.mockImplementation(() => Promise.resolve(replyLost ? configurationWithDebounce(debounce) : appConfigFixture));
  write.mockImplementation(() => {
    replyLost = true;
    return Promise.reject(failure);
  });

  function Settings(): ReactElement {
    const config = useAppConfig();
    const {submission, update} = useAppConfigCommands();
    useEffect(() => {
      observeMount('mount');
      return () => observeMount('unmount');
    }, []);

    return (
      <>
        <p>Confirmed: {config.popover.behavior.toggleDebounceMs}</p>
        {submission.status !== 'idle' && <p>Submitted: {submission.configuration.popover.behavior.toggleDebounceMs}</p>}
        {submission.status === 'failed' && submission.error instanceof ConfigurationCommandError && (
          <p>Submission error: {submission.error.code}</p>
        )}
        <button
          onClick={() =>
            update(draft => {
              draft.popover.behavior.toggleDebounceMs = Milliseconds(999);
            })
          }
          type='button'
        >
          Save settings
        </button>
      </>
    );
  }

  const view = await renderSettingsRouter(Settings);
  try {
    await expect.element(page.getByText('Confirmed: 300', {exact: true})).toBeVisible();
    await page.getByRole('button', {name: 'Save settings'}).click();
    await expect.element(page.getByText(failure.message, {exact: true})).toBeVisible();
    await expect.element(page.getByRole('button', {name: 'Save settings'})).not.toBeInTheDocument();
    await vi.waitFor(() => expect(observeMount.mock.calls).toEqual([['mount'], ['unmount']]));

    const readsBefore = read.mock.calls.length;
    await page.getByRole('button', {name: 'Try Again'}).click();
    await expect.element(page.getByText(`Confirmed: ${debounce}`, {exact: true})).toBeVisible();
    await expect.element(page.getByText('Submitted: 999', {exact: true})).toBeVisible();
    await expect.element(page.getByText('Submission error: result_unavailable', {exact: true})).toBeVisible();
    expect(read.mock.calls.length).toBeGreaterThan(readsBefore);
    expect(write.mock.calls).toEqual([[configurationWithDebounce(999)]]);
    await vi.waitFor(() => expect(observeMount.mock.calls).toEqual([['mount'], ['unmount'], ['mount']]));
  } finally {
    await view.unmount();
    read.mockReset();
    write.mockReset();
  }
});
