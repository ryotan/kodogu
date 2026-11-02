import {beforeEach, describe, expect, it, vi} from 'vitest';
import {renderHook} from 'vitest-browser-react';

import {QueryClient, QueryClientProvider} from '@tanstack/react-query';

import {create} from 'mutative';

import {setConfig} from '#commands/appConfig';
import {appConfigFixture} from '#commands/appConfig.fixture';
import {Milliseconds} from '#config';
import {appConfigQuery} from '#hooks/useAppConfig';
import {useAppConfigCommands} from '#hooks/useAppConfigCommands';
import {reportError} from '#reportError';

vi.mock('#commands/appConfig', async importOriginal => ({
  ...(await importOriginal<typeof import('#commands/appConfig')>()),
  setConfig: vi.fn().mockResolvedValue(undefined),
}));
vi.mock('#reportError', () => ({reportError: vi.fn()}));

const mockReportError = vi.mocked(reportError);
const mockSetConfig = vi.mocked(setConfig);

beforeEach(() => {
  mockSetConfig.mockClear();
  mockReportError.mockClear();
});

function createQueryClient() {
  const client = new QueryClient({
    defaultOptions: {queries: {retry: false}, mutations: {retry: false}},
  });
  client.setQueryData(appConfigQuery.queryKey, appConfigFixture);
  return client;
}

function renderCommands(client: QueryClient) {
  return renderHook(useAppConfigCommands, {
    wrapper: ({children}) => <QueryClientProvider client={client}>{children}</QueryClientProvider>,
  });
}

describe('useAppConfigCommands', () => {
  it('applies optimistic update to cache immediately', async () => {
    const client = createQueryClient();
    const {result, act} = await renderCommands(client);
    const next = create(appConfigFixture, draft => {
      draft.popover.behavior.toggleDebounceMs = Milliseconds(500);
    });

    await act(() =>
      result.current.update(draft => {
        draft.popover.behavior.toggleDebounceMs = Milliseconds(500);
      }),
    );

    expect(client.getQueryData(appConfigQuery.queryKey)).toEqual(next);
  });

  it('persists the updated config', async () => {
    const client = createQueryClient();
    const {result, act} = await renderCommands(client);
    const next = create(appConfigFixture, draft => {
      draft.popover.behavior.toggleDebounceMs = Milliseconds(999);
    });

    await act(() =>
      result.current.update(draft => {
        draft.popover.behavior.toggleDebounceMs = Milliseconds(999);
      }),
    );

    await vi.waitFor(() => {
      expect(mockSetConfig.mock.calls.map(([config]) => config)).toEqual([next]);
      expect(result.current.isPending).toBe(false);
    });
    expect(mockReportError).not.toHaveBeenCalled();
  });

  it('rolls back cache on mutation error', async () => {
    const mutationError = new Error('DB write failed');
    const setConfigCall: {reject?: (reason: Error) => void} = {};
    mockSetConfig.mockImplementationOnce(
      () =>
        new Promise((_resolve, reject) => {
          setConfigCall.reject = reject;
        }),
    );
    const client = createQueryClient();
    const {result, act} = await renderCommands(client);
    const next = create(appConfigFixture, draft => {
      draft.popover.behavior.toggleDebounceMs = Milliseconds(999);
    });

    await act(() =>
      result.current.update(draft => {
        draft.popover.behavior.toggleDebounceMs = Milliseconds(999);
      }),
    );

    // Reject only after checking the complete optimistic value, so recovery proves rollback rather than no update.
    await vi.waitFor(() => {
      expect(mockSetConfig.mock.calls.map(([config]) => config)).toEqual([next]);
    });
    expect(client.getQueryData(appConfigQuery.queryKey)).toEqual(next);
    const rejectPersistence = setConfigCall.reject;
    if (!rejectPersistence) {
      throw new Error('Persistence did not start');
    }
    await act(() => rejectPersistence(mutationError));

    await vi.waitFor(() => {
      expect(client.getQueryData(appConfigQuery.queryKey)).toEqual(appConfigFixture);
      expect(result.current.isPending).toBe(false);
    });
    expect(mockReportError.mock.calls).toEqual([['Failed to persist AppConfig', mutationError]]);
    expect(mockReportError.mock.calls[0][1]).toBe(mutationError);
  });

  it('throws when cache is not populated', async () => {
    const client = new QueryClient({
      defaultOptions: {queries: {retry: false}, mutations: {retry: false}},
    });
    const {result} = await renderCommands(client);

    expect(() =>
      result.current.update(draft => {
        draft.popover.behavior.toggleDebounceMs = Milliseconds(500);
      }),
    ).toThrow('update() called before AppConfig was loaded — this should be unreachable');
    expect(mockSetConfig).not.toHaveBeenCalled();
    expect(mockReportError).not.toHaveBeenCalled();
  });
});
