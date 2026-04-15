import {describe, expect, it, vi} from 'vitest';
import {page} from 'vitest/browser';
import {render} from 'vitest-browser-react';

import {QueryClient, QueryClientProvider, QueryErrorResetBoundary} from '@tanstack/react-query';

import {PopoverAppErrorPage} from './PopoverAppErrorPage';

const wrap = (ui: React.ReactElement) => {
  const queryClient = new QueryClient();
  return render(
    <QueryClientProvider client={queryClient}>
      <QueryErrorResetBoundary>{ui}</QueryErrorResetBoundary>
    </QueryClientProvider>,
  );
};

describe('PopoverAppErrorPage', () => {
  it('calls reset when Try Again is clicked', async () => {
    const mockReset = vi.fn();
    await wrap(<PopoverAppErrorPage error={new Error('test')} reset={mockReset} />);

    await page.getByRole('button', {name: /try again/i}).click();
    expect(mockReset).toHaveBeenCalledOnce();
  });

  it('shows component stack when info is provided', async () => {
    await wrap(
      <PopoverAppErrorPage
        error={new Error('test')}
        info={{componentStack: '\n    at SomeComponent\n    at RouterProvider'}}
        reset={vi.fn()}
      />,
    );

    await page.getByRole('button', {name: /component stack/i}).click();
    await expect.element(page.getByText(/SomeComponent/)).toBeVisible();
  });

  it('resets TanStack Query error boundary state when Try Again is clicked', async () => {
    // QueryErrorResetBoundary.reset marks the boundary as reset (isReset = true),
    // which allows suspended queries to re-throw and retry on the next render.
    // We verify this by using the render callback form of QueryErrorResetBoundary
    // to observe the isReset state before and after clicking Try Again.
    const queryClient = new QueryClient();

    // Use the render callback form to capture the boundary value.
    // The boundary value is a stable object reference, so we capture it once.
    let boundaryValue: {isReset: () => boolean; reset: () => void; clearReset: () => void} | null = null;

    await render(
      <QueryClientProvider client={queryClient}>
        <QueryErrorResetBoundary>
          {value => {
            boundaryValue = value;
            return <PopoverAppErrorPage error={new Error('test')} reset={vi.fn()} />;
          }}
        </QueryErrorResetBoundary>
      </QueryClientProvider>,
    );

    expect(boundaryValue).not.toBeNull();
    // biome-ignore lint/style/noNonNullAssertion: assigned by the render callback; not null after await render()
    expect(boundaryValue!.isReset()).toBe(false);

    await page.getByRole('button', {name: /try again/i}).click();

    // After clicking Try Again, the component called useQueryErrorResetBoundary().reset,
    // marking the boundary as reset so suspended queries can retry.
    // biome-ignore lint/style/noNonNullAssertion: assigned by the render callback; not null after await render()
    expect(boundaryValue!.isReset()).toBe(true);
  });
});
