import {describe, expect, it} from 'vitest';

import {formatOriginLocation, isExternalLocation} from './frameLocation';

// =============================================================================
// isExternalLocation
// =============================================================================

describe('isExternalLocation', () => {
  it('returns true for node_modules path', () => {
    expect(isExternalLocation('/app/node_modules/react/index.js')).toBe(true);
  });

  it('returns true for .vite/deps path', () => {
    expect(isExternalLocation('http://localhost:5173/.vite/deps/react.js')).toBe(true);
  });

  it('returns true when both node_modules and .vite/deps are present', () => {
    expect(isExternalLocation('/node_modules/.vite/deps/chunk.js')).toBe(true);
  });

  it('returns true for [native code]', () => {
    expect(isExternalLocation('[native code]')).toBe(true);
  });

  it('returns false for application source path', () => {
    expect(isExternalLocation('http://localhost:5173/src/hooks/useAppConfig.ts:7:14')).toBe(false);
  });

  it('returns false for empty string', () => {
    expect(isExternalLocation('')).toBe(false);
  });
});

// =============================================================================
// formatOriginLocation
// =============================================================================

describe('formatOriginLocation', () => {
  it('removes http origin and column number', () => {
    expect(formatOriginLocation('http://localhost:5173/src/hooks/useAppConfig.ts:7:14')).toBe(
      'src/hooks/useAppConfig.ts:7',
    );
  });

  it('removes https origin and column number', () => {
    expect(formatOriginLocation('https://example.com/src/main.tsx:42:5')).toBe('src/main.tsx:42');
  });

  it('removes column but keeps line when no origin', () => {
    expect(formatOriginLocation('src/hooks/useAppConfig.ts:7:14')).toBe('src/hooks/useAppConfig.ts:7');
  });

  it('preserves location when no column present', () => {
    expect(formatOriginLocation('src/main.tsx:42')).toBe('src/main.tsx:42');
  });

  it('preserves location when no line or column', () => {
    expect(formatOriginLocation('src/main.tsx')).toBe('src/main.tsx');
  });

  it('handles location with only origin (no path)', () => {
    expect(formatOriginLocation('http://localhost:5173/')).toBe('');
  });
});
