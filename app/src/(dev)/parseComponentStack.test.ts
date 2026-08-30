import {describe, expect, it} from 'vitest';

import {parseComponentStack} from './parseComponentStack';

describe('parseComponentStack', () => {
  it('parses a plain frame without location', () => {
    expect(parseComponentStack('    at App')).toEqual([{component: 'App'}]);
  });

  it('parses a frame with full file:line:col location', () => {
    expect(parseComponentStack('    at App (file.tsx:42:18)')).toEqual([
      {component: 'App', location: 'file.tsx:42:18'},
    ]);
  });

  it('accepts a column-less location (file:line)', () => {
    expect(parseComponentStack('    at App (file.tsx:42)')).toEqual([{component: 'App', location: 'file.tsx:42'}]);
  });

  it('keeps nested parens inside the component name (HOC pattern)', () => {
    expect(parseComponentStack('    at ForwardRef(MyComp) (src/foo.tsx:1:1)')).toEqual([
      {component: 'ForwardRef(MyComp)', location: 'src/foo.tsx:1:1'},
    ]);
  });

  it('handles multiple nested HOC wrappers', () => {
    expect(parseComponentStack('    at Memo(Lazy(MyComp))')).toEqual([{component: 'Memo(Lazy(MyComp))'}]);
  });

  it('returns an empty array for empty input', () => {
    expect(parseComponentStack('')).toEqual([]);
  });

  it('skips lines that do not match the React frame shape', () => {
    const stack = ['Error: boom', '    at App (file.tsx:42:18)', 'random debug output', '    at Root'].join('\n');
    expect(parseComponentStack(stack)).toEqual([{component: 'App', location: 'file.tsx:42:18'}, {component: 'Root'}]);
  });

  it('tolerates the leading newline that React emits', () => {
    const stack = '\n    at App\n    at Root';
    expect(parseComponentStack(stack)).toEqual([{component: 'App'}, {component: 'Root'}]);
  });
});
