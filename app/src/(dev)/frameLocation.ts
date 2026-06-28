/**
 * Stack-location helpers for the dev-only {@link import('./StackTrace').StackTrace} renderer:
 * classify (app vs external) and format frames that come out of `error-stack-parser`. Lives under
 * `(dev)/` because that is its only consumer and it is excluded from production bundles.
 */

export function isExternalLocation(location: string): boolean {
  // node_modules in production builds; .vite/deps in Vite dev server.
  return location.includes('node_modules') || location.includes('/.vite/deps/') || location === '[native code]';
}

// "http://localhost:5173/src/hooks/useAppConfig.ts:7:14" → "src/hooks/useAppConfig.ts:7"
export function formatOriginLocation(location: string): string {
  const withoutOrigin = location.replace(/^https?:\/\/[^/]+\//, '');
  return withoutOrigin.replace(/:(\d+):\d+$/, ':$1');
}
