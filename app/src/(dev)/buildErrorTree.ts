// Shapes a caught Error into a render-ready tree. A node's children (its "causes") are its
// `.cause` (0..1) concatenated with its aggregate `.errors` (0..N) — both are just underlying
// errors, differing only in count, so they are not distinguished here.
//
// name/message are intentionally not stored: they derive from `error`, and the renderer reads
// them directly so there is a single source of truth.

const MAX_DEPTH = 10;

export type ErrorNode = {
  error: Error;
  children: ErrorChild[];
};

export type ErrorChild =
  | {kind: 'error'; node: ErrorNode}
  | {kind: 'raw'; value: string}
  | {kind: 'cutoff'; reason: 'cycle' | 'max-depth'};

export function buildErrorTree(error: Error): ErrorNode {
  return build(error, new Set([error]), 0);
}

// `ancestors` holds the errors on the current path (not every error ever seen) so that a genuine
// cycle is cut while a non-cyclic error shared across branches stays visible on each branch.
function build(error: Error, ancestors: Set<Error>, depth: number): ErrorNode {
  return {error, children: collectCauses(error).map(cause => toChild(cause, ancestors, depth))};
}

function collectCauses(error: Error): unknown[] {
  const causes: unknown[] = [];
  if (error.cause !== undefined && error.cause !== null) {
    causes.push(error.cause);
  }
  if (error instanceof AggregateError) {
    causes.push(...error.errors);
  }
  return causes;
}

function toChild(cause: unknown, ancestors: Set<Error>, depth: number): ErrorChild {
  if (!(cause instanceof Error)) {
    return {kind: 'raw', value: String(cause)};
  }
  if (ancestors.has(cause)) {
    return {kind: 'cutoff', reason: 'cycle'};
  }
  if (depth + 1 >= MAX_DEPTH) {
    return {kind: 'cutoff', reason: 'max-depth'};
  }
  return {kind: 'error', node: build(cause, new Set(ancestors).add(cause), depth + 1)};
}
