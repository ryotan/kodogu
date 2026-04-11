export type StackFrame = {name: string; location: string; isExternal: boolean};

export function isExternalLocation(location: string): boolean {
  // node_modules in production builds; .vite/deps in Vite dev server
  return location.includes('node_modules') || location.includes('/.vite/deps/');
}

export function parseFrame(line: string): StackFrame | null {
  // WebKit: name@url or @url
  const webkit = /^([^@]*)@(.+)$/.exec(line);
  if (webkit) {
    const location = webkit[2]!;
    return {name: webkit[1] || '(anonymous)', location, isExternal: isExternalLocation(location)};
  }
  // V8: at name (url) or at url
  const v8 = /^\s*at\s+(?:(.+?)\s+\(?(.+?)\)?|(.+))$/.exec(line);
  if (v8) {
    const location = v8[2] || v8[3] || '';
    return {name: v8[1] || '(anonymous)', location, isExternal: isExternalLocation(location)};
  }
  return null;
}

// "http://localhost:5173/src/hooks/useAppConfig.ts:7:14" → "src/hooks/useAppConfig.ts:7"
export function formatOriginLocation(location: string): string {
  const withoutOrigin = location.replace(/^https?:\/\/[^/]+\//, '');
  return withoutOrigin.replace(/:(\d+):\d+$/, ':$1');
}

export function parseStackTrace(stack: string): StackFrame[] {
  return stack
    .split('\n')
    .slice(1) // First line is "ErrorName: message" — skip it
    .map(parseFrame)
    .filter((f): f is StackFrame => f !== null);
}
