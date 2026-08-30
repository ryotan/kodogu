import {RootError} from './RootError';

/**
 * Returns the message that should be shown to the user for `error`.
 *
 * Pattern A from ADR-0007: an instance of {@link RootError} (or a subclass —
 * {@link import('./ApplicationError').ApplicationError},
 * {@link import('./PlatformError').PlatformError}, etc.) is contracted to carry
 * a user-facing Japanese message in `error.message`, so it passes through
 * verbatim. Anything else falls back to a generic message — we don't want
 * unknown errors leaking developer-facing text (English exception strings,
 * native errors from `fetch`/Tauri/etc.) into the UI.
 */
export function getUserMessage(error: unknown): string {
  if (error instanceof RootError) {
    return error.message;
  }
  return '予期しないエラーが発生しました';
}
