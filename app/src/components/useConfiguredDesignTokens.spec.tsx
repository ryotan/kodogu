import {afterEach, describe, expect, it, vi} from 'vitest';
import {render} from 'vitest-browser-react';

import type {ReactElement} from 'react';

import {create} from 'mutative';

import {appConfigFixture} from '#commands/appConfig.fixture';
import {CssColor} from '#config';

const mockInvoke = vi.fn().mockResolvedValue(undefined);

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args) as unknown,
}));

const {QueryClient, QueryClientProvider} = await import('@tanstack/react-query');
const {appConfigQuery} = await import('#hooks/useAppConfig');
const {useConfiguredDesignTokens} = await import('#components/useConfiguredDesignTokens');

function DesignTokenConsumer(): ReactElement {
  useConfiguredDesignTokens();
  return <div data-testid='synced' />;
}

const cssVarNames = [
  '--kodogu-surface-color',
  '--kodogu-border-color',
  '--kodogu-backdrop-filter',
  '--kodogu-font-family',
  '--kodogu-header-height',
  '--kodogu-card-list-padding',
] as const;

describe('useConfiguredDesignTokens', () => {
  afterEach(() => {
    const root = document.documentElement;
    for (const name of cssVarNames) {
      root.style.removeProperty(name);
    }
  });

  it('sets CSS custom properties on document root after mount', async () => {
    const qc = new QueryClient({defaultOptions: {queries: {retry: false}}});
    qc.setQueryData(appConfigQuery.queryKey, appConfigFixture);

    await render(
      <QueryClientProvider client={qc}>
        <DesignTokenConsumer />
      </QueryClientProvider>,
    );

    const root = document.documentElement;
    const {appearance, layout} = appConfigFixture.popover;
    expect(root.style.getPropertyValue('--kodogu-surface-color')).toBe(appearance.surfaceColor);
    expect(root.style.getPropertyValue('--kodogu-border-color')).toBe(appearance.borderColor);
    expect(root.style.getPropertyValue('--kodogu-backdrop-filter')).toBe(appearance.backdropFilter);
    expect(root.style.getPropertyValue('--kodogu-font-family')).toBe(appearance.fontFamily);
    expect(root.style.getPropertyValue('--kodogu-header-height')).toBe(layout.headerHeight);
    expect(root.style.getPropertyValue('--kodogu-card-list-padding')).toBe(layout.cardListPadding);
  });

  it('updates CSS custom properties when config changes', async () => {
    const qc = new QueryClient({defaultOptions: {queries: {retry: false}}});
    qc.setQueryData(appConfigQuery.queryKey, appConfigFixture);

    await render(
      <QueryClientProvider client={qc}>
        <DesignTokenConsumer />
      </QueryClientProvider>,
    );

    // Update cached config
    qc.setQueryData(
      appConfigQuery.queryKey,
      create(appConfigFixture, draft => {
        draft.popover.appearance.surfaceColor = CssColor('rgba(0, 0, 0, 1)');
      }),
    );

    await vi.waitFor(() => {
      expect(document.documentElement.style.getPropertyValue('--kodogu-surface-color')).toBe('rgba(0, 0, 0, 1)');
    });
  });
});
