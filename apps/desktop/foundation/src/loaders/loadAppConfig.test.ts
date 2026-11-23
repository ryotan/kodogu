import {beforeEach, describe, expect, it, vi} from 'vitest';

import {QueryClient} from '@tanstack/react-query';
import {createMemoryHistory, createRouter} from '@tanstack/react-router';

import {invoke} from '@tauri-apps/api/core';

import {appConfigQuery} from '#hooks/useAppConfig';

import {Route} from './__root';

vi.mock('@tauri-apps/api/core', () => ({invoke: vi.fn()}));

const mockInvoke = vi.mocked(invoke);

// The loader hands whatever `get_config` returned straight to the cache, so an opaque payload states
// what is under test more clearly than a full AppConfig fixture would.
const configFromIpc = {marker: 'app-config'};

// The root route is mounted on its own — its loader is the only one that has to run here.
const loadRootRoute = (queryClient: QueryClient) =>
  createRouter({
    routeTree: Route,
    context: {queryClient},
    history: createMemoryHistory({initialEntries: ['/']}),
  }).load();

describe('root route loader', () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    mockInvoke.mockResolvedValue(configFromIpc);
  });

  it('loads the app config into the cache', async () => {
    const queryClient = new QueryClient();

    await loadRootRoute(queryClient);

    expect(mockInvoke).toHaveBeenCalledWith('get_config');
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
    expect(mockInvoke).toHaveBeenCalledTimes(1);
  });
});
