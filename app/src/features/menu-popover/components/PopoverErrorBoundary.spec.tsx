import {afterEach, beforeEach, describe, expect, it, vi} from 'vitest';
import {page} from 'vitest/browser';
import {render} from 'vitest-browser-react';

import {QueryClient, QueryClientProvider} from '@tanstack/react-query';

import {PopoverErrorBoundary} from './PopoverErrorBoundary';

// React error boundaries always call console.error in test environments — suppress it
beforeEach(() => {
  vi.spyOn(console, 'error').mockImplementation(vi.fn());
});
afterEach(() => {
  vi.restoreAllMocks();
});

const ThrowError = () => {
  throw new Error('boundary test error');
};

const wrap = (ui: React.ReactElement) => {
  const queryClient = new QueryClient();
  return render(<QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>);
};

describe('PopoverErrorBoundary', () => {
  it('renders fallback when child throws', async () => {
    await wrap(
      <PopoverErrorBoundary>
        <ThrowError />
      </PopoverErrorBoundary>,
    );

    await expect.element(page.getByRole('button', {name: /try again/i})).toBeVisible();
  });

  it('recovers after Try Again when child no longer throws', async () => {
    let shouldThrow = true;
    const ConditionalThrow = () => {
      if (shouldThrow) {
        throw new Error('test');
      }
      return <div>recovered</div>;
    };

    await wrap(
      <PopoverErrorBoundary>
        <ConditionalThrow />
      </PopoverErrorBoundary>,
    );

    await expect.element(page.getByRole('button', {name: /try again/i})).toBeVisible();

    shouldThrow = false;
    await page.getByRole('button', {name: /try again/i}).click();

    await expect.element(page.getByText('recovered')).toBeVisible();
  });
});
