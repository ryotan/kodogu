import {beforeEach, describe, expect, it, vi} from 'vitest';
import {renderHook} from 'vitest-browser-react';

import type {ReactElement, ReactNode} from 'react';

import {QueryClient, QueryClientProvider} from '@tanstack/react-query';

import {create} from 'mutative';

import {getConfig, setConfig} from '#commands/appConfig';
import {appConfigFixture} from '#commands/appConfig.fixture';
import {ConfigurationCommandError} from '#commands/ConfigurationCommandError';
import type {AppConfig} from '#config';
import {LogicalPixel, Milliseconds} from '#config';
import {AppConfigCommandsProvider} from '#hooks/AppConfigCommandsProvider';
import {appConfigQuery} from '#hooks/useAppConfig';
import {useAppConfigCommands} from '#hooks/useAppConfigCommands';
import {reportError} from '#reportError';

vi.mock('#commands/appConfig', () => ({getConfig: vi.fn(), setConfig: vi.fn()}));
vi.mock('#reportError', () => ({reportError: vi.fn()}));
const mockSetConfig = vi.mocked(setConfig);
const mockGetConfig = vi.mocked(getConfig);
const mockReportError = vi.mocked(reportError);

beforeEach(() => {
  mockSetConfig.mockReset();
  mockGetConfig.mockReset();
  mockReportError.mockReset();
});

function client(): QueryClient {
  const queryClient = new QueryClient({defaultOptions: {queries: {retry: false}, mutations: {retry: false}}});
  queryClient.setQueryData(appConfigQuery.queryKey, appConfigFixture);
  return queryClient;
}

function wrapper(queryClient: QueryClient): ({children}: {children: ReactNode}) => ReactElement {
  function Wrapper({children}: {children: ReactNode}): ReactElement {
    return (
      <QueryClientProvider client={queryClient}>
        <AppConfigCommandsProvider>{children}</AppConfigCommandsProvider>
      </QueryClientProvider>
    );
  }
  return Wrapper;
}

function changed(value: number): AppConfig {
  return create(appConfigFixture, draft => {
    draft.popover.behavior.toggleDebounceMs = Milliseconds(value);
  });
}

describe('useAppConfigCommands', () => {
  it('composes edits from independent callers before either write completes', async () => {
    let complete: (() => void) | undefined;
    mockSetConfig
      .mockImplementationOnce(
        () =>
          new Promise<void>(resolve => {
            complete = resolve;
          }),
      )
      .mockResolvedValue(undefined);
    const queryClient = client();
    const {result, act} = await renderHook(() => ({first: useAppConfigCommands(), second: useAppConfigCommands()}), {
      wrapper: wrapper(queryClient),
    });
    const expected = create(changed(500), draft => {
      draft.popover.layout.initialWidth = LogicalPixel(640);
    });
    await act(() => {
      expect(
        result.current.first.update(draft => {
          draft.popover.behavior.toggleDebounceMs = Milliseconds(500);
        }),
      ).toEqual({status: 'submitted'});
      expect(
        result.current.second.update(draft => {
          draft.popover.layout.initialWidth = LogicalPixel(640);
        }),
      ).toEqual({status: 'submitted'});
    });
    await vi.waitFor(() => expect(mockSetConfig.mock.calls).toEqual([[changed(500)]]));
    expect(result.current.first.submission).toEqual({status: 'pending', configuration: expected});
    expect(result.current.second.submission).toEqual({status: 'pending', configuration: expected});
    expect(queryClient.getQueryData(appConfigQuery.queryKey)).toEqual(appConfigFixture);
    await act(() => {
      complete?.();
    });
    await vi.waitFor(() => expect(result.current.second.submission).toEqual({status: 'idle'}));
    expect(queryClient.getQueryData(appConfigQuery.queryKey)).toEqual(expected);
    expect(mockSetConfig.mock.calls).toEqual([[changed(500)], [expected]]);
  });

  it('publishes settings only after their write succeeds', async () => {
    let complete: (() => void) | undefined;
    mockSetConfig.mockImplementation(
      () =>
        new Promise<void>(resolve => {
          complete = resolve;
        }),
    );
    const queryClient = client();
    const {result, act} = await renderHook(() => useAppConfigCommands(), {wrapper: wrapper(queryClient)});
    await act(() => {
      result.current.update(draft => {
        draft.popover.behavior.toggleDebounceMs = Milliseconds(500);
      });
    });
    await vi.waitFor(() => expect(mockSetConfig.mock.calls).toEqual([[changed(500)]]));
    expect(result.current.submission).toEqual({status: 'pending', configuration: changed(500)});
    expect(queryClient.getQueryData(appConfigQuery.queryKey)).toEqual(appConfigFixture);
    await act(() => {
      complete?.();
    });
    await vi.waitFor(() => expect(result.current.submission).toEqual({status: 'idle'}));
    expect(queryClient.getQueryData(appConfigQuery.queryKey)).toEqual(changed(500));
    expect(mockSetConfig.mock.calls).toEqual([[changed(500)]]);
  });

  it('keeps confirmed settings and submitted input after a known rejection', async () => {
    const failure = new ConfigurationCommandError('configuration_rejected');
    mockSetConfig.mockRejectedValue(failure);
    const queryClient = client();
    const {result, act} = await renderHook(() => useAppConfigCommands(), {wrapper: wrapper(queryClient)});
    await act(() => {
      result.current.update(draft => {
        draft.popover.behavior.toggleDebounceMs = Milliseconds(999);
      });
    });
    await vi.waitFor(() =>
      expect(result.current.submission).toEqual({status: 'failed', configuration: changed(999), error: failure}),
    );
    expect(queryClient.getQueryData(appConfigQuery.queryKey)).toEqual(appConfigFixture);
    expect(queryClient.getQueryState(appConfigQuery.queryKey)?.status).toBe('success');
    expect(mockSetConfig.mock.calls).toEqual([[changed(999)]]);
    expect(mockReportError.mock.calls).toEqual([['configuration_write_failed', failure]]);
    expect(mockReportError.mock.calls[0]?.[1]).toBe(failure);
  });

  it('saves an independent queued edit without carrying over a rejected field or discarding its input', async () => {
    const failure = new ConfigurationCommandError('configuration_rejected');
    let rejectFirst: (() => void) | undefined;
    mockSetConfig
      .mockImplementationOnce(
        () =>
          new Promise<void>((_resolve, reject) => {
            rejectFirst = () => reject(failure);
          }),
      )
      .mockImplementation(configuration =>
        configuration.popover.layout.initialWidth <= 0 ? Promise.reject(failure) : Promise.resolve(),
      );
    const queryClient = client();
    const {result, act} = await renderHook(() => ({first: useAppConfigCommands(), second: useAppConfigCommands()}), {
      wrapper: wrapper(queryClient),
    });
    const rejected = create(appConfigFixture, draft => {
      draft.popover.layout.initialWidth = LogicalPixel(-1);
    });
    const saved = create(appConfigFixture, draft => {
      draft.popover.layout.bottomMargin = LogicalPixel(12);
    });
    await act(() => {
      result.current.first.update(draft => {
        draft.popover.layout.initialWidth = LogicalPixel(-1);
      });
      result.current.second.update(draft => {
        draft.popover.layout = {...draft.popover.layout, bottomMargin: LogicalPixel(12)};
      });
    });
    await vi.waitFor(() => expect(mockSetConfig.mock.calls).toEqual([[rejected]]));
    expect(queryClient.getQueryData(appConfigQuery.queryKey)).toEqual(appConfigFixture);
    await act(() => {
      rejectFirst?.();
    });
    await vi.waitFor(() =>
      expect(result.current.second.submission).toEqual({status: 'failed', configuration: rejected, error: failure}),
    );
    expect(result.current.first.submission).toEqual({status: 'failed', configuration: rejected, error: failure});
    expect(mockSetConfig.mock.calls).toEqual([[rejected], [saved]]);
    expect(queryClient.getQueryData(appConfigQuery.queryKey)).toEqual(saved);
    expect(mockReportError.mock.calls).toEqual([['configuration_write_failed', failure]]);
  });

  it.each([
    {code: 'configuration_unavailable', failure: new ConfigurationCommandError('configuration_unavailable')},
    {code: 'result_unavailable', failure: new ConfigurationCommandError('result_unavailable')},
  ])('marks current settings unavailable and retains input after $code', async ({failure}) => {
    mockSetConfig.mockRejectedValue(failure);
    const queryClient = client();
    const {result, act} = await renderHook(() => useAppConfigCommands(), {wrapper: wrapper(queryClient)});
    await act(() => {
      result.current.update(draft => {
        draft.popover.behavior.toggleDebounceMs = Milliseconds(999);
      });
    });
    await vi.waitFor(() =>
      expect(result.current.submission).toEqual({status: 'failed', configuration: changed(999), error: failure}),
    );
    expect(queryClient.getQueryState(appConfigQuery.queryKey)?.status).toBe('error');
    expect(queryClient.getQueryState(appConfigQuery.queryKey)?.error).toBe(failure);
    const recipe = vi.fn();
    expect(result.current.update(recipe)).toEqual({status: 'unavailable', error: failure});
    expect(recipe).not.toHaveBeenCalled();
    expect(mockSetConfig.mock.calls).toEqual([[changed(999)]]);
  });

  it('composes consecutive input while serializing persistence', async () => {
    let complete: (() => void) | undefined;
    mockSetConfig
      .mockImplementationOnce(
        () =>
          new Promise<void>(resolve => {
            complete = resolve;
          }),
      )
      .mockResolvedValue(undefined);
    const queryClient = client();
    const {result, act} = await renderHook(() => useAppConfigCommands(), {wrapper: wrapper(queryClient)});
    await act(() => {
      result.current.update(draft => {
        draft.popover.behavior.toggleDebounceMs = Milliseconds(500);
      });
      result.current.update(draft => {
        draft.popover.behavior.toggleDebounceMs = Milliseconds(draft.popover.behavior.toggleDebounceMs + 1);
      });
    });
    await vi.waitFor(() => expect(mockSetConfig).toHaveBeenCalledOnce());
    await act(() => {
      complete?.();
    });
    await vi.waitFor(() => expect(queryClient.getQueryData(appConfigQuery.queryKey)).toEqual(changed(501)));
    await vi.waitFor(() => expect(result.current.submission).toEqual({status: 'idle'}));
    expect(mockSetConfig.mock.calls).toEqual([[changed(500)], [changed(501)]]);
  });

  it('does not send queued writes after the first result becomes unavailable', async () => {
    const failure = new ConfigurationCommandError('configuration_unavailable');
    mockSetConfig.mockRejectedValue(failure);
    const queryClient = client();
    const {result, act} = await renderHook(() => useAppConfigCommands(), {wrapper: wrapper(queryClient)});
    await act(() => {
      result.current.update(draft => {
        draft.popover.behavior.toggleDebounceMs = Milliseconds(500);
      });
      result.current.update(draft => {
        draft.popover.behavior.toggleDebounceMs = Milliseconds(501);
      });
    });
    await vi.waitFor(() => expect(queryClient.getQueryState(appConfigQuery.queryKey)?.status).toBe('error'));
    await vi.waitFor(() =>
      expect(result.current.submission).toEqual({
        status: 'not_sent',
        configuration: changed(501),
        blockedBy: {status: 'unavailable', error: failure},
      }),
    );
    expect(mockSetConfig.mock.calls).toEqual([[changed(500)]]);
    expect(mockReportError.mock.calls).toEqual([['configuration_write_failed', failure]]);
  });

  it('starts a new edit from recovered settings without sending edits queued before the lost reply', async () => {
    const failure = new ConfigurationCommandError('result_unavailable');
    mockSetConfig.mockRejectedValueOnce(failure).mockResolvedValue(undefined);
    const queryClient = client();
    const recovered = changed(700);
    mockReportError.mockImplementationOnce(() => {
      // Publish a newer confirmed snapshot before the mutation scope resumes queued writes.
      queryClient.setQueryData(appConfigQuery.queryKey, recovered);
    });
    const {result, act} = await renderHook(() => useAppConfigCommands(), {wrapper: wrapper(queryClient)});
    await act(() => {
      result.current.update(draft => {
        draft.popover.behavior.toggleDebounceMs = Milliseconds(500);
      });
      result.current.update(draft => {
        draft.popover.behavior.toggleDebounceMs = Milliseconds(501);
      });
    });
    await vi.waitFor(() =>
      expect(result.current.submission).toEqual({
        status: 'not_sent',
        configuration: changed(501),
        blockedBy: {status: 'unavailable', error: failure},
      }),
    );
    expect(queryClient.getQueryData(appConfigQuery.queryKey)).toEqual(recovered);
    expect(mockSetConfig.mock.calls).toEqual([[changed(500)]]);

    const expected = create(recovered, draft => {
      draft.popover.layout.initialWidth = LogicalPixel(640);
    });
    await act(() => {
      expect(
        result.current.update(draft => {
          draft.popover.layout.initialWidth = LogicalPixel(640);
        }),
      ).toEqual({status: 'submitted'});
    });
    await vi.waitFor(() => expect(result.current.submission).toEqual({status: 'idle'}));
    expect(queryClient.getQueryData(appConfigQuery.queryKey)).toEqual(expected);
    expect(mockSetConfig.mock.calls).toEqual([[changed(500)], [expected]]);
    expect(mockReportError.mock.calls).toEqual([['configuration_write_failed', failure]]);
  });

  it('rejects updates before settings have been loaded', async () => {
    const queryClient = new QueryClient();
    const {result} = await renderHook(() => useAppConfigCommands(), {wrapper: wrapper(queryClient)});
    const recipe = vi.fn();
    expect(result.current.update(recipe)).toEqual({status: 'not_loaded'});
    expect(result.current.submission).toEqual({status: 'idle'});
    expect(recipe).not.toHaveBeenCalled();
    expect(mockSetConfig).not.toHaveBeenCalled();
    expect(mockReportError).not.toHaveBeenCalled();
  });
});

it('does not overwrite newly committed settings with an older in-flight read', async () => {
  let completeRead: ((configuration: AppConfig) => void) | undefined;
  mockGetConfig.mockImplementation(
    () =>
      new Promise<AppConfig>(resolve => {
        completeRead = resolve;
      }),
  );
  mockSetConfig.mockResolvedValue(undefined);
  const queryClient = client();
  const read = queryClient.query({...appConfigQuery, staleTime: 0}).catch((error: unknown) => error);
  await vi.waitFor(() => expect(mockGetConfig).toHaveBeenCalledOnce());
  const {result, act} = await renderHook(() => useAppConfigCommands(), {wrapper: wrapper(queryClient)});
  await act(() => {
    result.current.update(draft => {
      draft.popover.behavior.toggleDebounceMs = Milliseconds(500);
    });
  });
  await vi.waitFor(() => expect(queryClient.getQueryData(appConfigQuery.queryKey)).toEqual(changed(500)));
  await act(() => {
    completeRead?.(appConfigFixture);
  });
  await read;
  expect(queryClient.getQueryData(appConfigQuery.queryKey)).toEqual(changed(500));
  expect(mockSetConfig.mock.calls).toEqual([[changed(500)]]);
});
