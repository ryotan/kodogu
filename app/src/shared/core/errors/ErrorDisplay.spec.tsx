import {describe, expect, it} from 'vitest';
import {page} from 'vitest/browser';
import {render} from 'vitest-browser-react';

import {ErrorDisplay} from './ErrorDisplay';

describe('ErrorDisplay', () => {
  it('renders error name and message', async () => {
    const error = new Error('something went wrong');
    error.name = 'TestError';

    await render(<ErrorDisplay error={error} />);

    await expect.element(page.getByText('TestError')).toBeVisible();
    await expect.element(page.getByText('something went wrong')).toBeVisible();
  });

  it('renders single Error cause recursively', async () => {
    const cause = new Error('root cause');
    const error = new Error('outer', {cause});

    await render(<ErrorDisplay error={error} />);

    await expect.element(page.getByText('outer')).toBeVisible();
    await expect.element(page.getByText('root cause')).toBeVisible();
  });

  it('renders AggregateError cause errors as a list', async () => {
    const agg = new AggregateError([new Error('child 1'), new Error('child 2')], 'parallel failures');
    const error = new Error('outer', {cause: agg});

    await render(<ErrorDisplay error={error} />);

    await expect.element(page.getByText('outer')).toBeVisible();
    await expect.element(page.getByText('parallel failures')).toBeVisible();
    await expect.element(page.getByText('child 1')).toBeVisible();
    await expect.element(page.getByText('child 2')).toBeVisible();
  });

  it('stops recursing into cause chain at MAX_CAUSE_DEPTH', async () => {
    // Build a chain longer than MAX_CAUSE_DEPTH (10)
    let deepest = new Error('depth-11');
    for (let i = 10; i >= 1; i--) {
      const e = new Error(`depth-${i}`);
      e.cause = deepest;
      deepest = e;
    }
    const root = new Error('depth-0', {cause: deepest});

    await render(<ErrorDisplay error={root} />);

    await expect.element(page.getByText('depth-0')).toBeVisible();
    await expect.element(page.getByText('depth-10')).toBeVisible();
    // depth-11 exceeds MAX_CAUSE_DEPTH and must not be rendered
    await expect.element(page.getByText('depth-11')).not.toBeInTheDocument();
  });

  it('omits the AggregateError card when depth limit is reached', async () => {
    // Build a deep chain whose terminal cause is an AggregateError. The Card for the
    // AggregateError appears at depth MAX_CAUSE_DEPTH (10), so it must be suppressed.
    const aggAtDepth11 = new AggregateError([new Error('agg-child')], 'agg at limit');
    let chain: Error = aggAtDepth11;
    for (let i = 10; i >= 1; i--) {
      chain = new Error(`depth-${i}`, {cause: chain});
    }
    const root = new Error('depth-0', {cause: chain});

    await render(<ErrorDisplay error={root} />);

    await expect.element(page.getByText('depth-10')).toBeVisible();
    // The AggregateError sits beyond MAX_CAUSE_DEPTH, so neither its message nor its
    // children should appear.
    await expect.element(page.getByText('agg at limit')).not.toBeInTheDocument();
    await expect.element(page.getByText('agg-child')).not.toBeInTheDocument();
  });

  it('skips non-Error items in AggregateError.errors', async () => {
    // errors array contains a mix of Error and non-Error values
    const agg = new AggregateError([new Error('real error'), 'string value'], 'mixed');
    const error = new Error('outer', {cause: agg});

    await render(<ErrorDisplay error={error} />);

    await expect.element(page.getByText('real error')).toBeVisible();
    await expect.element(page.getByText('string value')).not.toBeInTheDocument();
  });

  it('renders AggregateError child errors when cause has no message', async () => {
    // AggregateError without a message — cause.message is '' (falsy), so no message line rendered
    const agg = new AggregateError([new Error('child')]);
    const error = new Error('outer', {cause: agg});

    await render(<ErrorDisplay error={error} />);

    await expect.element(page.getByText('outer')).toBeVisible();
    await expect.element(page.getByText('child')).toBeVisible();
  });

  it('does not infinitely recurse on a circular cause chain', async () => {
    const a = new Error('a');
    const b = new Error('b');
    // Create circular reference: a → b → a
    a.cause = b;
    Object.defineProperty(b, 'cause', {value: a, writable: true, configurable: true});

    // Should terminate without throwing (depth guard stops the cycle)
    await expect(render(<ErrorDisplay error={a} />)).resolves.not.toThrow();
  });
});
