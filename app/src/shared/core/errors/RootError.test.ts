import {describe, expect, it} from 'vitest';

import {RootError} from './RootError';

describe('RootError', () => {
  it('sets name to constructor name for direct instances', () => {
    const error = new RootError('test');
    expect(error.name).toBe('RootError');
  });

  it('sets name to subclass name automatically', () => {
    class MyError extends RootError {}
    const error = new MyError('test');
    expect(error.name).toBe('MyError');
  });

  it('is an instance of Error', () => {
    const error = new RootError('test');
    expect(error).toBeInstanceOf(Error);
  });

  it('passes cause to Error options', () => {
    const cause = new Error('original');
    const error = new RootError('wrapped', {cause});
    expect(error.cause).toBe(cause);
  });
});
