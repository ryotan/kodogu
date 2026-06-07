/**
 * Base class for every error thrown inside the application.
 *
 * Set `this.name` to the actual constructor name so that subclasses surface their own type
 * in stack traces and logs without having to override `name` themselves.
 *
 * Production code should not throw `RootError` directly — pick {@link ApplicationError} or
 * {@link PlatformError} depending on the source of the failure, or extend one of them for
 * domain-specific cases. `RootError` itself exists only to anchor `instanceof` checks at the
 * top of the hierarchy and to centralize the `name` plumbing.
 */
export class RootError extends Error {
  constructor(message: string, options?: ErrorOptions) {
    super(message, options);
    this.name = this.constructor.name;
  }
}
