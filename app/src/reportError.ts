/**
 * Preserves existing logging calls pending the frontend error-handling design.
 * The message/details API is provisional and may be revised as part of that design.
 */
export function reportError(message: string, ...details: readonly unknown[]): void {
  console.error(message, ...details);
}
