import {describe, expect, it} from 'vitest';

import {TypedAggregateError} from './TypedAggregateError';

describe('TypedAggregateError', () => {
  it('is an instance of AggregateError', () => {
    const error = new TypedAggregateError([new Error('a')], 'test');
    expect(error).toBeInstanceOf(AggregateError);
  });

  it('has name TypedAggregateError', () => {
    const error = new TypedAggregateError([new Error('a')], 'test');
    expect(error.name).toBe('TypedAggregateError');
  });

  it('exposes typed errors array', () => {
    class MyError extends Error {}
    const errors = [new MyError('a'), new MyError('b')];
    const aggregate = new TypedAggregateError(errors, 'two errors');
    expect(aggregate.errors).toHaveLength(2);
    expect(aggregate.errors[0]).toBeInstanceOf(MyError);
    expect(aggregate.errors[1]).toBeInstanceOf(MyError);
  });

  it('works as cause in another error', () => {
    const errors = [new Error('x')];
    const cause = new TypedAggregateError(errors, 'aggregate');
    const outer = new Error('outer', {cause});
    expect(outer.cause).toBeInstanceOf(AggregateError);
    if (!(outer.cause instanceof AggregateError)) {
      throw new Error('Expected AggregateError');
    }
    expect(outer.cause.errors[0]).toBeInstanceOf(Error);
  });
});
