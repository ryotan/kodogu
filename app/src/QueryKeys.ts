/**
 * Centralized React Query cache keys for the entire application.
 *
 * All query keys live here, so the shared cache structure is visible in one place.
 * Each entry is a function, so subkeys can be derived with spread when needed.
 */
export const QueryKeys = {
  appConfig: () => ['app-config'] as const,
} as const;
