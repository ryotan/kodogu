import {describe, expect, it} from 'vitest';

import {TypedAggregateError} from '#shared/core/errors/TypedAggregateError';

import type {ErrorChild} from './buildErrorTree';
import {buildErrorTree} from './buildErrorTree';

// Narrowing helper: fail loudly when a child is not the expected `error` kind so the assertions
// below can read `node` directly.
const expectErrorChild = (child: ErrorChild) => {
  if (child.kind !== 'error') {
    throw new Error(`expected an error child, got ${child.kind}`);
  }
  return child.node;
};

describe('buildErrorTree', () => {
  it('returns a leaf node for an error without causes', () => {
    const error = new Error('boom');

    const tree = buildErrorTree(error);

    expect(tree.error).toBe(error);
    expect(tree.children).toEqual([]);
  });

  it('nests a single .cause as a child node', () => {
    const cause = new Error('root cause');
    const error = new Error('outer', {cause});

    const tree = buildErrorTree(error);

    expect(tree.children).toHaveLength(1);
    expect(expectErrorChild(tree.children[0]).error).toBe(cause);
  });

  it('lists aggregate .errors as parallel children', () => {
    const a = new Error('a');
    const b = new Error('b');
    const error = new TypedAggregateError([a, b], 'two failures');

    const tree = buildErrorTree(error);

    expect(tree.children.map(c => expectErrorChild(c).error)).toEqual([a, b]);
  });

  it('concatenates .cause and aggregate .errors with the cause first', () => {
    const cause = new Error('cause');
    const member = new Error('member');
    const error = new TypedAggregateError([member], 'agg', {cause});

    const tree = buildErrorTree(error);

    expect(tree.children.map(c => expectErrorChild(c).error)).toEqual([cause, member]);
  });

  it('descends into an aggregate that appears as a cause', () => {
    const inner = new Error('inner');
    const aggregate = new TypedAggregateError([inner], 'agg');
    const error = new Error('outer', {cause: aggregate});

    const tree = buildErrorTree(error);

    const aggregateNode = expectErrorChild(tree.children[0]);
    expect(aggregateNode.error).toBe(aggregate);
    expect(expectErrorChild(aggregateNode.children[0]).error).toBe(inner);
  });

  it('represents a non-Error cause as a raw child', () => {
    const error = new Error('outer', {cause: 'literal cause'});

    const tree = buildErrorTree(error);

    expect(tree.children).toEqual([{kind: 'raw', value: 'literal cause'}]);
  });

  it('ignores a null or undefined cause', () => {
    const error = new Error('outer', {cause: null});

    const tree = buildErrorTree(error);

    expect(tree.children).toEqual([]);
  });

  it('cuts off a cyclic cause chain at the re-entry point', () => {
    const a = new Error('a');
    const b = new Error('b');
    a.cause = b;
    b.cause = a;

    const tree = buildErrorTree(a);

    const bNode = expectErrorChild(tree.children[0]);
    expect(bNode.children).toEqual([{kind: 'cutoff', reason: 'cycle'}]);
  });

  it('keeps a shared, non-cyclic error visible on every path', () => {
    const shared = new Error('shared');
    const error = new TypedAggregateError([new Error('a', {cause: shared}), new Error('b', {cause: shared})], 'agg');

    const tree = buildErrorTree(error);

    const [a, b] = tree.children.map(expectErrorChild);
    expect(expectErrorChild(a.children[0]).error).toBe(shared);
    expect(expectErrorChild(b.children[0]).error).toBe(shared);
  });

  it('cuts off a cause chain that exceeds the depth limit', () => {
    let current = new Error('leaf');
    for (let i = 0; i < 20; i += 1) {
      current = new Error(`level-${i}`, {cause: current});
    }

    const tree = buildErrorTree(current);

    let node = tree;
    let cutoff: ErrorChild | undefined;
    while (node.children.length > 0) {
      const child = node.children[0];
      if (child.kind === 'cutoff') {
        cutoff = child;
        break;
      }
      node = expectErrorChild(child);
    }
    expect(cutoff).toEqual({kind: 'cutoff', reason: 'max-depth'});
  });
});
