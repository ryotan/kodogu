import {beforeEach, describe, expect, it, vi} from 'vitest';
import {render} from 'vitest-browser-react';

import {useEffect} from 'react';

import type {Draft} from 'mutative';

import {defaultAppConfig} from '#commands/appConfig.fixture';
import type {AppConfig} from '#config';
import {Milliseconds} from '#config';

const mockInvoke = vi.fn().mockResolvedValue(undefined);

beforeEach(() => {
  mockInvoke.mockClear();
});

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args) as unknown,
}));

const {QueryClient, QueryClientProvider} = await import('@tanstack/react-query');
const {appConfigQuery} = await import('#hooks/useAppConfig');
const {useAppConfigCommands} = await import('#hooks/useAppConfigCommands');

function createQueryClient() {
  const qc = new QueryClient({
    defaultOptions: {queries: {retry: false}, mutations: {retry: false}},
  });
  qc.setQueryData(appConfigQuery.queryKey, defaultAppConfig);
  return qc;
}

type UpdateFn = (recipe: (draft: Draft<AppConfig>) => void) => void;
const captured: {update: UpdateFn | null} = {update: null};

const TestComponent = () => {
  const {update} = useAppConfigCommands();
  useEffect(() => {
    captured.update = update;
  }, [update]);
  return <div data-testid='ready' />;
};

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
        config: {
          ...defaultAppConfig,
          popover: {
            ...defaultAppConfig.popover,
            behavior: {toggleDebounceMs: 999},
          },
        },
      });
    });
  });

  it('rolls back cache on mutation error', async () => {
    mockInvoke.mockRejectedValueOnce(new Error('DB write failed'));
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
      const cached = qc.getQueryData(appConfigQuery.queryKey);
      expect(cached?.popover.behavior.toggleDebounceMs).toBe(300);
    });
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
