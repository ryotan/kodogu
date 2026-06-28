/**
 * `AggregateError` with a typed `errors` array. JS's built-in `AggregateError.errors` is `any[]`,
 * which loses the element type; this subclass narrows it to `E[]` so callers iterating the children
 * keep full type information. Represents several failures collected together (e.g. multiple
 * form-validation errors). Like the rest of the hierarchy it sets `name` to the constructor name.
 *
 * Note: it extends `AggregateError`, not {@link import('./RootError').RootError}, so it does not
 * itself carry a Pattern-A user message — wrap it in a `RootError` that carries the message, or rely
 * on {@link import('./getUserMessage').getUserMessage}'s generic fallback.
 */
export class TypedAggregateError<E extends Error> extends AggregateError {
  // Override AggregateError.errors (typed as any[]) with typed elements.
  // `declare` emits no runtime code — super() sets the actual value.
  // Cannot use `ReadonlyArray<E>` here: TypeScript disallows narrowing a mutable
  // base property (any[]) to a readonly subtype in a subclass override.
  declare errors: E[];

  constructor(errors: readonly E[], message?: string, options?: ErrorOptions) {
    super(errors, message, options);
    this.name = this.constructor.name;
  }
}
