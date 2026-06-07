import {RootError} from './RootError';

/**
 * Errors that originate from application logic — domain rule violations, use case
 * preconditions, input validation failures, and invariants that the app itself owns.
 *
 * Throw `ApplicationError` (or extend it) when the cause is expressible in app-domain terms:
 * "config not yet loaded", "connection name too long", "tried to update before mount", etc.
 * The thing went wrong _because of what the app expected_, not because something underneath
 * misbehaved.
 *
 * For failures rooted in the runtime environment (Tauri IPC, filesystem, OS), reach for
 * {@link PlatformError} instead.
 */
export class ApplicationError extends RootError {}
