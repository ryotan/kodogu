import {describe, expect, it, vi} from 'vitest';
import {render} from 'vitest-browser-react';

import {AppErrorComponent} from './AppErrorComponent';
import styles from './AppErrorComponent.module.css';

type FrameName = 'repeat' | 'caller';
type StackFormat = 'v8' | 'webkit';
type FrameFixture = Readonly<{v8: string; webkit: string; text: string}>;

const frames: Readonly<Record<FrameName, FrameFixture>> = {
  repeat: {
    v8: '    at repeat (http://localhost:5173/src/repeat.ts:10:4)',
    webkit: 'repeat@http://localhost:5173/src/repeat.ts:10:4',
    text: 'repeat @ http://localhost:5173/src/repeat.ts:10:4',
  },
  caller: {
    v8: '    at caller (http://localhost:5173/src/caller.ts:20:8)',
    webkit: 'caller@http://localhost:5173/src/caller.ts:20:8',
    text: 'caller @ http://localhost:5173/src/caller.ts:20:8',
  },
};

const cases: ReadonlyArray<{
  readonly name: string;
  readonly before: readonly FrameName[];
  readonly after: readonly FrameName[];
  readonly retainsRows?: boolean;
}> = [
  {
    name: 'unchanged repeated frames',
    before: ['repeat', 'repeat', 'caller'],
    after: ['repeat', 'repeat', 'caller'],
    retainsRows: true,
  },
  {
    name: 'reordered repeated frames',
    before: ['repeat', 'repeat', 'caller'],
    after: ['caller', 'repeat', 'repeat'],
  },
  {
    name: 'fewer repetitions',
    before: ['repeat', 'repeat', 'repeat', 'caller'],
    after: ['caller', 'repeat'],
  },
  {
    name: 'more repetitions',
    before: ['caller', 'repeat'],
    after: ['repeat', 'repeat', 'repeat', 'caller'],
  },
  {
    name: 'only the caller',
    before: ['repeat', 'repeat', 'caller'],
    after: ['caller'],
  },
  {
    name: 'an empty stack',
    before: ['repeat', 'repeat', 'caller'],
    after: [],
  },
];
const renderCases: readonly Readonly<{format: StackFormat; unparsedLines: boolean}>[] = [
  {format: 'v8', unparsedLines: false},
  {format: 'webkit', unparsedLines: false},
  {format: 'v8', unparsedLines: true},
  {format: 'webkit', unparsedLines: true},
];

function createError(names: readonly FrameName[], format: StackFormat, unparsedLines: boolean): Error {
  const error = new Error('Stack update');
  const lines = names.map(name => frames[name][format]);
  error.stack = ['Error: Stack update', ...(unparsedLines ? ['unrecognized stack metadata'] : []), ...lines].join('\n');
  return error;
}

describe.each(renderCases)(
  'AppErrorComponent $format stack (unparsed lines: $unparsedLines)',
  ({format, unparsedLines}) => {
    it.each(cases)('renders $name after updating the error page', async ({before, after, retainsRows}) => {
      const reset = vi.fn<() => void>();
      const initialError = createError(before, format, unparsedLines);
      const result = await render(<AppErrorComponent error={initialError} reset={reset} />);
      const rowNodes = () => Array.from(result.container.getElementsByClassName(styles.frameRow));
      const readRows = () => rowNodes().map(row => row.textContent);

      expect(readRows()).toEqual(before.map(name => frames[name].text));
      await result.getByRole('button', {name: 'Stack trace'}).click();
      const initialRows = rowNodes();
      const nextError = retainsRows ? initialError : createError(after, format, unparsedLines);

      await result.rerender(<AppErrorComponent error={nextError} reset={reset} />);

      await expect.poll(readRows, {timeout: 1000}).toEqual(after.map(name => frames[name].text));
      if (retainsRows) {
        rowNodes().forEach((row, index) => {
          expect(row).toBe(initialRows[index]);
        });
      }
      expect(reset).not.toHaveBeenCalled();
    });
  },
);
