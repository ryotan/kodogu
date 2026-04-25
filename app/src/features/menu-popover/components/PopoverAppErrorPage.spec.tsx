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

  // ComponentStack is dev-only; its devtools-mode rendering is verified in
  // PopoverAppErrorPage.devtools.spec.tsx (browser-devtools project).

  it('resets TanStack Query error boundary state when Try Again is clicked', async () => {
    // QueryErrorResetBoundary.reset marks the boundary as reset (isReset = true),
    // which allows suspended queries to re-throw and retry on the next render.
    // We verify this by using the render callback form of QueryErrorResetBoundary
    // to observe the isReset state before and after clicking Try Again.
    const queryClient = new QueryClient();

    // Use the render callback form to observe the isReset state.
    // Each render updates boundaryValue, so it always holds the latest state.
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
    // biome-ignore lint/style/noNonNullAssertion: proven non-null by the expect above; TypeScript cannot narrow through expect()
    const boundary = boundaryValue!;
    expect(boundary.isReset()).toBe(false);

    await page.getByRole('button', {name: /try again/i}).click();

    // After clicking Try Again, the component called useQueryErrorResetBoundary().reset,
    // marking the boundary as reset so suspended queries can retry.
    expect(boundary.isReset()).toBe(true);
  });
});
