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
