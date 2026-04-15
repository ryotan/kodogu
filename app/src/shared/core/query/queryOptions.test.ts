import {describe, expect, it} from 'vitest';

import type {QueryFunctionContext, QueryKey} from '@tanstack/react-query';
import {skipToken} from '@tanstack/react-query';

import {QueryCallSiteError, queryOptions} from './queryOptions';

// Minimal QueryFunctionContext for regular (non-infinite) queries.
// Cast to bypass required fields (e.g. `client`) that our wrapper never accesses.
const makeContext = <K extends QueryKey>(queryKey: K): QueryFunctionContext<K> =>
  ({queryKey, signal: new AbortController().signal, meta: undefined}) as unknown as QueryFunctionContext<K>;

describe('queryOptions', () => {
  it('attaches QueryCallSiteError as cause when queryFn throws without cause', async () => {
    const key = ['test', 123] as const;
    const options = queryOptions({
      queryKey: key,
      queryFn: () => {
        throw new Error('original');
      },
    });

    await expect(options.queryFn(makeContext(key))).rejects.toMatchObject({
      message: 'original',
      cause: {
        queryKey: ['test', 123],
        message: 'Query called with key: ["test",123]',
      },
    });
  });

  it('cause is an instance of QueryCallSiteError with correct name', async () => {
    const key = ['test'] as const;
    const options = queryOptions({
      queryKey: key,
      queryFn: () => {
        throw new Error('original');
      },
    });

    const thrown = await options.queryFn(makeContext(key)).catch((e: unknown) => e);
    if (!(thrown instanceof Error)) {
      throw new Error(`Expected Error, got ${typeof thrown}`);
    }
    expect(thrown.cause).toBeInstanceOf(QueryCallSiteError);
    if (!(thrown.cause instanceof QueryCallSiteError)) {
      throw new Error('Expected QueryCallSiteError');
    }
    expect(thrown.cause.name).toBe('QueryCallSiteError');
  });

  it('does not overwrite existing cause', async () => {
    const key = ['test'] as const;
    const existingCause = new Error('existing cause');
    const options = queryOptions({
      queryKey: key,
      queryFn: () => {
        throw new Error('original', {cause: existingCause});
      },
    });

    await expect(options.queryFn(makeContext(key))).rejects.toMatchObject({
      cause: existingCause,
    });
  });

  it('does not wrap undefined queryFn as a function', () => {
    // The wrapper passes non-function queryFn through unchanged.
    // TanStack Query returns queryFn: undefined when omitted — we verify the wrapper
    // doesn't accidentally replace it with a wrapped function.
    const options = queryOptions({queryKey: ['test']});
    expect(typeof options.queryFn).not.toBe('function');
  });

  it('passes through when queryFn is skipToken', () => {
    // skipToken is a truthy symbol — typeof !== 'function' correctly passes it through
    const options = queryOptions({queryKey: ['test'], queryFn: skipToken});
    expect(options.queryFn).toBe(skipToken);
  });
});
