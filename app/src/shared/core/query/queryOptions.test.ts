import {skipToken} from '@tanstack/react-query';
import type {QueryFunctionContext, QueryKey} from '@tanstack/react-query';
import {describe, expect, it} from 'vitest';

import {QueryCallSiteError, queryOptions} from './queryOptions';

// Minimal QueryFunctionContext for regular (non-infinite) queries.
// Generic parameter prevents type errors with narrow `as const` query keys.
const makeContext = <K extends QueryKey>(queryKey: K): QueryFunctionContext<K> => ({
  queryKey,
  signal: new AbortController().signal,
  meta: undefined,
});

describe('queryOptions', () => {
  it('attaches QueryCallSiteError as cause when queryFn throws without cause', async () => {
    const key = ['test', 123] as const;
    const options = queryOptions({
      queryKey: key,
      queryFn: async () => { throw new Error('original'); },
    });

    await expect(options.queryFn!(makeContext(key))).rejects.toMatchObject({
      message: 'original',
      cause: expect.objectContaining({
        queryKey: ['test', 123],
        message: 'Query called with key: ["test",123]',
      }),
    });
  });

  it('cause is an instance of QueryCallSiteError with correct name', async () => {
    const key = ['test'] as const;
    const options = queryOptions({
      queryKey: key,
      queryFn: async () => { throw new Error('original'); },
    });

    const error = await options.queryFn!(makeContext(key)).catch(e => e);
    expect(error.cause).toBeInstanceOf(QueryCallSiteError);
    expect(error.cause.name).toBe('QueryCallSiteError');
  });

  it('does not overwrite existing cause', async () => {
    const key = ['test'] as const;
    const existingCause = new Error('existing cause');
    const options = queryOptions({
      queryKey: key,
      queryFn: async () => { throw new Error('original', {cause: existingCause}); },
    });

    await expect(options.queryFn!(makeContext(key))).rejects.toMatchObject({
      cause: existingCause,
    });
  });

  it('passes through when queryFn is not provided', () => {
    const options = queryOptions({queryKey: ['test']});
    expect(options.queryFn).toBeUndefined();
  });

  it('passes through when queryFn is skipToken', () => {
    // skipToken is a truthy symbol — typeof !== 'function' correctly passes it through
    const options = queryOptions({queryKey: ['test'], queryFn: skipToken});
    expect(options.queryFn).toBe(skipToken);
  });
});
