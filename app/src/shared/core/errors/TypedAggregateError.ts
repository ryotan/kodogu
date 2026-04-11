export class TypedAggregateError<E extends Error> extends AggregateError {
  // Override AggregateError.errors (typed as any[]) with a typed readonly array.
  // `declare` emits no runtime code — super() sets the actual value.
  declare readonly errors: ReadonlyArray<E>;

  constructor(errors: ReadonlyArray<E>, message?: string, options?: ErrorOptions) {
    super(errors, message, options);
    this.name = 'TypedAggregateError';
  }
}
