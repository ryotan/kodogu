import {beforeEach, describe, expect, it, vi} from 'vitest';
import {render} from 'vitest-browser-react';

import type {ReactElement} from 'react';
import {useEffect} from 'react';

import {QueryClient, QueryClientProvider} from '@tanstack/react-query';

import type {Draft} from 'mutative';
import {create} from 'mutative';

import {appConfigFixture} from '#commands/appConfig.fixture';
import type {AppConfig} from '#config';
import {Milliseconds} from '#config';
import {appConfigQuery} from '#hooks/useAppConfig';
import {useAppConfigCommands} from '#hooks/useAppConfigCommands';
import {reportError} from '#reportError';

const {mockInvoke} = vi.hoisted(() => ({mockInvoke: vi.fn().mockResolvedValue(undefined)}));

beforeEach(() => {
  mockInvoke.mockClear();
  mockReportError.mockClear();
});

vi.mock('@tauri-apps/api/core', () => ({invoke: mockInvoke}));
vi.mock('#reportError', () => ({reportError: vi.fn()}));

const mockReportError = vi.mocked(reportError);

function createQueryClient() {
  const qc = new QueryClient({
    defaultOptions: {queries: {retry: false}, mutations: {retry: false}},
  });
  qc.setQueryData(appConfigQuery.queryKey, appConfigFixture);
  return qc;
}

type UpdateFn = (recipe: (draft: Draft<AppConfig>) => void) => void;
const captured: {update: UpdateFn | null} = {update: null};

function TestComponent(): ReactElement {
  const {update} = useAppConfigCommands();
  useEffect(() => {
    captured.update = update;
  }, [update]);
  return <div data-testid='ready' />;
}

describe('useAppConfigCommands', () => {
  beforeEach(() => {
    captured.update = null;
  });

  it('applies optimistic update to cache immediately', async () => {
    const qc = createQueryClient();
    await render(
      <QueryClientProvider client={qc}>
        <TestComponent />
      </QueryClientProvider>,
    );

    captured.update?.(draft => {
      draft.popover.behavior.toggleDebounceMs = Milliseconds(500);
    });

    await vi.waitFor(() => {
      const cached = qc.getQueryData(appConfigQuery.queryKey);
      expect(cached?.popover.behavior.toggleDebounceMs).toBe(500);
    });
  });

  it('calls set_config via invoke with the new config', async () => {
    const qc = createQueryClient();
    await render(
      <QueryClientProvider client={qc}>
        <TestComponent />
      </QueryClientProvider>,
    );

    captured.update?.(draft => {
      draft.popover.behavior.toggleDebounceMs = Milliseconds(999);
    });

    await vi.waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('set_config', {
        config: create(appConfigFixture, draft => {
          draft.popover.behavior.toggleDebounceMs = Milliseconds(999);
        }),
      });
    });
  });

  it('rolls back cache on mutation error', async () => {
    const mutationError = new Error('DB write failed');
    const setConfigCall: {reject?: (reason: Error) => void} = {};
    mockInvoke.mockImplementationOnce(
      () =>
        new Promise((_resolve, reject) => {
          setConfigCall.reject = reject;
        }),
    );
    const qc = createQueryClient();
    await render(
      <QueryClientProvider client={qc}>
        <TestComponent />
      </QueryClientProvider>,
    );

    captured.update?.(draft => {
      draft.popover.behavior.toggleDebounceMs = Milliseconds(999);
    });

    // Fails `set_config` only after the optimistic value is in the cache, so the old value can only come back from
    // the rollback.
    await vi.waitFor(() => {
      expect(setConfigCall.reject).toBeDefined();
    });
    expect(qc.getQueryData(appConfigQuery.queryKey)?.popover.behavior.toggleDebounceMs).toBe(999);
    setConfigCall.reject?.(mutationError);

    await vi.waitFor(() => {
      const cached = qc.getQueryData(appConfigQuery.queryKey);
      expect(cached?.popover.behavior.toggleDebounceMs).toBe(appConfigFixture.popover.behavior.toggleDebounceMs);
    });
    expect(mockReportError).toHaveBeenCalledTimes(1);
    expect(mockReportError).toHaveBeenCalledWith('Failed to persist AppConfig', mutationError);
  });

  it('throws when cache is not populated', async () => {
    const qc = new QueryClient({
      defaultOptions: {queries: {retry: false}, mutations: {retry: false}},
    });
    await render(
      <QueryClientProvider client={qc}>
        <TestComponent />
      </QueryClientProvider>,
    );

    expect(() => {
      captured.update?.(draft => {
        draft.popover.behavior.toggleDebounceMs = Milliseconds(500);
      });
    }).toThrow('update() called before AppConfig was loaded');
  });
});
