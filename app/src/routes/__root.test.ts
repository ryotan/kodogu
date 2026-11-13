import {beforeEach, describe, expect, it, vi} from 'vitest';

import {QueryClient} from '@tanstack/react-query';
import {createMemoryHistory, createRouter} from '@tanstack/react-router';

import {create} from 'mutative';

import {getConfig} from '#commands/appConfig';
import {appConfigFixture} from '#commands/appConfig.fixture';
import {Milliseconds} from '#config';
import {appConfigQuery} from '#hooks/useAppConfig';

import {Route} from './__root';

vi.mock('#commands/appConfig', () => ({getConfig: vi.fn()}));
const mockGetConfig = vi.mocked(getConfig);

const configFromIpc = appConfigFixture;

// The root route is mounted on its own — its loader is the only one that has to run here.
const loadRootRoute = (queryClient: QueryClient) =>
  createRouter({
    routeTree: Route,
    context: {queryClient},
    history: createMemoryHistory({initialEntries: ['/']}),
  }).load();

describe('root route loader', () => {
  beforeEach(() => {
    mockGetConfig.mockReset();
    mockGetConfig.mockResolvedValue(configFromIpc);
  });

  it('loads the app config into the cache', async () => {
    const queryClient = new QueryClient();

    await loadRootRoute(queryClient);

    expect(mockGetConfig).toHaveBeenCalledOnce();
    expect(queryClient.getQueryData(appConfigQuery.queryKey)).toEqual(configFromIpc);
  });

  it('keeps serving the cached config after the query has been invalidated', async () => {
    const queryClient = new QueryClient();
    await loadRootRoute(queryClient);
    await queryClient.invalidateQueries({queryKey: appConfigQuery.queryKey});

    // A fresh router always runs its loaders — the test above is what guarantees that — so a single
    // fetch here means the loader ran and chose the cache, rather than never running at all.
    await loadRootRoute(queryClient);

    expect(queryClient.getQueryData(appConfigQuery.queryKey)).toEqual(configFromIpc);
    expect(mockGetConfig).toHaveBeenCalledTimes(1);
  });
});

it('requests a fresh settings read when retrying unavailable cached settings', async () => {
  mockGetConfig.mockReset();
  mockGetConfig.mockResolvedValue(configFromIpc);
  const queryClient = new QueryClient();
  await loadRootRoute(queryClient);
  queryClient
    .getQueryCache()
    .find({queryKey: appConfigQuery.queryKey})
    ?.setState({status: 'error', error: new Error('Configuration unavailable'), isInvalidated: true});
  const recovered = create(appConfigFixture, draft => {
    draft.popover.behavior.toggleDebounceMs = Milliseconds(999);
  });
  mockGetConfig.mockResolvedValueOnce(recovered);
  await loadRootRoute(queryClient);
  expect(mockGetConfig).toHaveBeenCalledTimes(2);
  expect(queryClient.getQueryData(appConfigQuery.queryKey)).toEqual(recovered);
  expect(queryClient.getQueryState(appConfigQuery.queryKey)?.status).toBe('success');
});
