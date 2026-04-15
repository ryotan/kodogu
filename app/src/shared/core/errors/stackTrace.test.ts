import {describe, expect, it} from 'vitest';

import {formatOriginLocation, isExternalLocation, parseFrame, parseStackTrace} from './stackTrace';

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

  it('returns false for application source path', () => {
    expect(isExternalLocation('http://localhost:5173/src/hooks/useAppConfig.ts:7:14')).toBe(false);
  });

  it('returns false for empty string', () => {
    expect(isExternalLocation('')).toBe(false);
  });
});

// =============================================================================
// parseFrame
// =============================================================================

describe('parseFrame', () => {
  describe('WebKit format (name@url)', () => {
    it('parses named frame', () => {
      const result = parseFrame('useAppConfig@http://localhost:5173/src/hooks/useAppConfig.ts:7:14');
      expect(result).toEqual({
        name: 'useAppConfig',
        location: 'http://localhost:5173/src/hooks/useAppConfig.ts:7:14',
        isExternal: false,
      });
    });

    it('parses anonymous frame (@url only)', () => {
      const result = parseFrame('@http://localhost:5173/src/main.tsx:42:5');
      expect(result).toEqual({
        name: '(anonymous)',
        location: 'http://localhost:5173/src/main.tsx:42:5',
        isExternal: false,
      });
    });

    it('marks node_modules frame as external', () => {
      const result = parseFrame('renderWithHooks@http://localhost:5173/node_modules/react-dom/client.js:123:10');
      expect(result).toEqual({
        name: 'renderWithHooks',
        location: 'http://localhost:5173/node_modules/react-dom/client.js:123:10',
        isExternal: true,
      });
    });

    it('marks .vite/deps frame as external', () => {
      const result = parseFrame('init@http://localhost:5173/.vite/deps/react.js?v=abc:1:2');
      expect(result).toEqual({
        name: 'init',
        location: 'http://localhost:5173/.vite/deps/react.js?v=abc:1:2',
        isExternal: true,
      });
    });
  });

  describe('V8 format (at name (url))', () => {
    it('parses named frame with parenthesized url', () => {
      const result = parseFrame('    at useAppConfig (http://localhost:5173/src/hooks/useAppConfig.ts:7:14)');
      expect(result).toEqual({
        name: 'useAppConfig',
        location: 'http://localhost:5173/src/hooks/useAppConfig.ts:7:14',
        isExternal: false,
      });
    });

    it('parses url-only frame (no name)', () => {
      const result = parseFrame('    at http://localhost:5173/src/main.tsx:42:5');
      expect(result).toEqual({
        name: '(anonymous)',
        location: 'http://localhost:5173/src/main.tsx:42:5',
        isExternal: false,
      });
    });

    it('marks node_modules frame as external', () => {
      const result = parseFrame(
        '    at renderWithHooks (http://localhost:5173/node_modules/react-dom/client.js:123:10)',
      );
      expect(result).toEqual({
        name: 'renderWithHooks',
        location: 'http://localhost:5173/node_modules/react-dom/client.js:123:10',
        isExternal: true,
      });
    });

    it('handles leading whitespace variations', () => {
      const result = parseFrame('  at foo (http://localhost:5173/src/foo.ts:1:1)');
      expect(result).toEqual({
        name: 'foo',
        location: 'http://localhost:5173/src/foo.ts:1:1',
        isExternal: false,
      });
    });
  });

  describe('non-matching input', () => {
    it('returns null for plain text', () => {
      expect(parseFrame('Error: something went wrong')).toBeNull();
    });

    it('returns null for empty string', () => {
      expect(parseFrame('')).toBeNull();
    });

    it('returns null for whitespace-only string', () => {
      expect(parseFrame('   ')).toBeNull();
    });
  });

  describe('V8 edge cases', () => {
    it('parses method frame with dot notation (Object.method)', () => {
      const result = parseFrame('    at Object.method (http://localhost:5173/src/foo.ts:1:1)');
      expect(result).toEqual({
        name: 'Object.method',
        location: 'http://localhost:5173/src/foo.ts:1:1',
        isExternal: false,
      });
    });

    it('treats "async" keyword as function name for V8 async frames (known limitation)', () => {
      // V8 emits "at async fn (url)" — the regex matches "async" as name and
      // "fn (url" as location. This is a known limitation; Tauri uses WKWebKit,
      // which emits "fn@url" and does not have this issue.
      const result = parseFrame('    at async fetchData (http://localhost:5173/src/foo.ts:5:3)');
      expect(result).not.toBeNull();
      expect(result?.name).toBe('async');
    });
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

// =============================================================================
// parseStackTrace
// =============================================================================

describe('parseStackTrace', () => {
  it('strips the first line and parses WebKit stack frames', () => {
    const stack = [
      'Error: test',
      'level3@http://localhost:5173/src/(dev)/StackTraceInspector.tsx:7:18',
      'executeDispatch@http://localhost:5173/node_modules/.vite/deps/react-dom_client.js:13622:19',
    ].join('\n');
    const frames = parseStackTrace(stack);
    expect(frames).toHaveLength(2);
    expect(frames[0].name).toBe('level3');
    expect(frames[0].isExternal).toBe(false);
    expect(frames[1].name).toBe('executeDispatch');
    expect(frames[1].isExternal).toBe(true);
  });

  it('strips the first line and parses V8 stack frames', () => {
    const stack = [
      'Error: test',
      '    at level3 (http://localhost:5173/src/foo.tsx:7:18)',
      '    at chunk (http://localhost:5173/node_modules/.vite/deps/react.js:1:1)',
    ].join('\n');
    const frames = parseStackTrace(stack);
    expect(frames).toHaveLength(2);
    expect(frames[0].name).toBe('level3');
    expect(frames[0].isExternal).toBe(false);
    expect(frames[1].name).toBe('chunk');
    expect(frames[1].isExternal).toBe(true);
  });

  it('returns empty array for empty string', () => {
    expect(parseStackTrace('')).toEqual([]);
  });

  it('returns empty array when stack has only the header line', () => {
    expect(parseStackTrace('Error: something went wrong')).toEqual([]);
  });

  it('filters out non-parseable lines such as empty lines', () => {
    const stack = [
      'Error: test',
      'level3@http://localhost:5173/src/foo.tsx:7:18',
      '',
      'renderWithHooks@http://localhost:5173/node_modules/react-dom/client.js:1:1',
    ].join('\n');
    const frames = parseStackTrace(stack);
    expect(frames).toHaveLength(2);
    expect(frames[0].name).toBe('level3');
    expect(frames[1].name).toBe('renderWithHooks');
  });
});
