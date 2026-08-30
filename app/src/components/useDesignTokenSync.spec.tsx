import {afterEach, describe, expect, it, vi} from 'vitest';
import {render} from 'vitest-browser-react';

import {createAppConfig} from '#commands/appConfig.fixture';
import {CssColor} from '#config';

const {mockInvoke} = vi.hoisted(() => ({
  mockInvoke: vi.fn().mockResolvedValue(undefined),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args) as unknown,
}));

import {QueryClient, QueryClientProvider} from '@tanstack/react-query';

import {useConfiguredDesignTokens} from '#components/useDesignTokenSync';
import {appConfigQuery} from '#hooks/useAppConfig';

function DesignTokenConsumer() {
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
    qc.setQueryData(appConfigQuery.queryKey, createAppConfig());

    await render(
      <QueryClientProvider client={qc}>
        <DesignTokenConsumer />
      </QueryClientProvider>,
    );

    const root = document.documentElement;
    expect(root.style.getPropertyValue('--kodogu-surface-color')).toBe('rgba(28, 28, 30, 0.92)');
    expect(root.style.getPropertyValue('--kodogu-border-color')).toBe('rgba(255, 255, 255, 0.08)');
    expect(root.style.getPropertyValue('--kodogu-backdrop-filter')).toBe('blur(24px) saturate(180%)');
    expect(root.style.getPropertyValue('--kodogu-font-family')).toBe(
      "-apple-system, BlinkMacSystemFont, 'Helvetica Neue', sans-serif",
    );
    expect(root.style.getPropertyValue('--kodogu-header-height')).toBe('40px');
    expect(root.style.getPropertyValue('--kodogu-card-list-padding')).toBe('6px 0');
  });

  it('updates CSS custom properties when config changes', async () => {
    const qc = new QueryClient({defaultOptions: {queries: {retry: false}}});
    qc.setQueryData(appConfigQuery.queryKey, createAppConfig());

    await render(
      <QueryClientProvider client={qc}>
        <DesignTokenConsumer />
      </QueryClientProvider>,
    );

    // Update cached config
    qc.setQueryData(appConfigQuery.queryKey, createAppConfig({surfaceColor: CssColor('rgba(0, 0, 0, 1)')}));

    await vi.waitFor(() => {
      expect(document.documentElement.style.getPropertyValue('--kodogu-surface-color')).toBe('rgba(0, 0, 0, 1)');
    });
  });
});
