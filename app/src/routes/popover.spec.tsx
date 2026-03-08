import {describe, expect, it, vi} from 'vitest';
import {page} from 'vitest/browser';
import {render} from 'vitest-browser-react';

const mockInvoke = vi.fn().mockResolvedValue(undefined);

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args) as unknown,
}));

const {Route} = await import('./popover');
const PopoverLayout = Route.options.component;
if (!PopoverLayout) {
  throw new Error('PopoverLayout component is not defined');
}

describe('PopoverLayout', () => {
  it('renders header, card list, and footer sections', async () => {
    await render(<PopoverLayout />);

    // Header: connection count summary is rendered
    await expect.element(page.getByText('2接続中 / 4プロファイル')).toBeVisible();

    // Card list: section labels from StubCardList are rendered
    await expect.element(page.getByText('接続中', {exact: true})).toBeVisible();

    // Footer: Quit button from StubFooter is rendered
    await expect.element(page.getByRole('button', {name: 'Quit'})).toBeVisible();
  });
});
