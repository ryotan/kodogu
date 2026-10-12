import {beforeEach, describe, expect, it, vi} from 'vitest';
import {render} from 'vitest-browser-react';

import {useEffect} from 'react';

import type {Draft} from 'mutative';

import type {AppConfig} from '#config';
import {CssColor, CssFilter, CssFontFamily, CssLength, CssPadding, LogicalPixel, Milliseconds} from '#config';

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

const defaultConfig: AppConfig = {
  popover: {
    behavior: {toggleDebounceMs: Milliseconds(300)},
    layout: {
      initialWidth: LogicalPixel(480),
      bottomMargin: LogicalPixel(8),
      trayGap: LogicalPixel(4),
      fallbackMaxHeight: LogicalPixel(600),
      minMaxHeight: LogicalPixel(200),
      headerHeight: CssLength('40px'),
      cardListPadding: CssPadding('6px 0'),
    },
    appearance: {
      surfaceColor: CssColor('rgba(28, 28, 30, 0.92)'),
      borderColor: CssColor('rgba(255, 255, 255, 0.08)'),
      backdropFilter: CssFilter('blur(24px) saturate(180%)'),
      fontFamily: CssFontFamily("-apple-system, BlinkMacSystemFont, 'Helvetica Neue', sans-serif"),
    },
  },
};

function createQueryClient() {
  const qc = new QueryClient({
    defaultOptions: {queries: {retry: false}, mutations: {retry: false}},
  });
  qc.setQueryData(appConfigQuery.queryKey, defaultConfig);
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
          ...defaultConfig,
          popover: {
            ...defaultConfig.popover,
            behavior: {toggleDebounceMs: 999},
          },
        },
      });
    });
  });

  it('rolls back cache on mutation error', async () => {
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
    setConfigCall.reject?.(new Error('DB write failed'));

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
