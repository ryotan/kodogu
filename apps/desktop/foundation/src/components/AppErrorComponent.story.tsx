import {useState} from 'react';

import type {Meta, StoryObj} from '@storybook/react-vite';
import {expect, fn, userEvent, waitFor, within} from 'storybook/test';

import {AppErrorComponent} from './AppErrorComponent';
import styles from './AppErrorComponent.module.css';

function createError(message: string, options?: {name?: string; stack?: string}): Error {
  const error = new Error(message);
  if (options?.name) {
    error.name = options.name;
  }
  if (options?.stack) {
    error.stack = options.stack;
  }
  return error;
}

const meta = {
  component: AppErrorComponent,
  parameters: {layout: 'fullscreen'},
  args: {
    error: new Error('Something went wrong'),
    reset: fn(),
  },
} satisfies Meta<typeof AppErrorComponent>;

export default meta;
type Story = StoryObj<typeof meta>;

// -- Stack trace fixtures --

const v8Stack = `Error: Failed to fetch config
    at useAppConfig (http://localhost:5173/src/hooks/useAppConfig.ts:7:14)
    at renderWithHooks (http://localhost:5173/node_modules/react-dom/client.js:123:10)
    at mountIndeterminateComponent (http://localhost:5173/node_modules/react-dom/client.js:456:7)
    at beginWork (http://localhost:5173/node_modules/react-dom/client.js:789:3)`;

const webkitStack = `TypeError: Cannot read properties of undefined
renderWithHooks@http://localhost:5173/node_modules/react-dom/client.js:123:10
mountIndeterminateComponent@http://localhost:5173/node_modules/react-dom/client.js:456:7
beginWork@http://localhost:5173/.vite/deps/react-dom.js?v=abc:789:3`;

const mixedStack = `Error: Something broke
    at PopoverLayout (http://localhost:5173/src/routes/popover.tsx:22:5)
    at useAutoResize (http://localhost:5173/src/hooks/useAutoResize.ts:45:12)
    at renderWithHooks (http://localhost:5173/node_modules/react-dom/client.js:123:10)
    at beginWork (http://localhost:5173/node_modules/react-dom/client.js:789:3)`;

// -- Stories --

export const BasicError: Story = {
  play: async ({canvasElement, args}) => {
    const canvas = within(canvasElement);

    await userEvent.click(canvas.getByRole('button', {name: /try again/i}));
    await expect(args.reset).toHaveBeenCalled();
  },
};

export const WithStackTrace: Story = {
  args: {
    error: createError('Failed to fetch config', {stack: v8Stack}),
  },
};

export const AllExternalFrames: Story = {
  args: {
    error: createError('Cannot read properties of undefined', {name: 'TypeError', stack: webkitStack}),
  },
};

export const MixedFrames: Story = {
  args: {
    error: createError('Something broke', {stack: mixedStack}),
  },
};

export const LongErrorMessage: Story = {
  args: {
    error: createError(
      'Failed to connect to the backend service at https://api.example.com/v1/config. ' +
        'The server might be down or the network connection is unstable. ' +
        'Please check your internet connection and try again. ' +
        'If the problem persists, contact support with error code ERR_CONNECTION_REFUSED.',
      {name: 'NetworkError'},
    ),
  },
};

export const ExpandCollapseStackTrace: Story = {
  args: {
    error: createError('Something broke', {stack: mixedStack}),
  },
  play: async ({canvasElement}) => {
    const canvas = within(canvasElement);
    const expandButton = canvas.getByRole('button', {name: /stack trace/i});

    // Collapsed by default
    const stackBody = canvasElement.querySelector('[class*="stackBody"]');
    await expect(stackBody).not.toBeNull();
    await expect(stackBody).not.toHaveAttribute('data-expanded');

    // Expand — verify both user-code and external frames are rendered
    await userEvent.click(expandButton);
    await expect(stackBody).toHaveAttribute('data-expanded');
    await expect(canvas.getByText('PopoverLayout')).toBeInTheDocument();
    await expect(canvas.getByText('useAutoResize')).toBeInTheDocument();
    await expect(canvas.getByText('renderWithHooks')).toBeInTheDocument();

    // Collapse
    await userEvent.click(expandButton);
    await expect(stackBody).not.toHaveAttribute('data-expanded');
  },
};

function ErrorTransition({initialError, nextError, reset}: {initialError: Error; nextError: Error; reset: () => void}) {
  // A parent state update preserves the child tree so frame-key regressions remain observable.
  const [updated, setUpdated] = useState(false);
  return (
    <>
      <button onClick={() => setUpdated(true)} type='button'>
        Update error
      </button>
      <AppErrorComponent error={updated ? nextError : initialError} reset={reset} />
    </>
  );
}

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

type FrameUpdateName = 'unchanged' | 'reordered' | 'fewer' | 'more' | 'callerOnly' | 'empty';
type FrameUpdate = Readonly<{
  name: string;
  before: readonly FrameName[];
  after: readonly FrameName[];
  retainsRows?: boolean;
}>;

const frameUpdates: Readonly<Record<FrameUpdateName, FrameUpdate>> = {
  unchanged: {
    name: 'unchanged repeated frames',
    before: ['repeat', 'repeat', 'caller'],
    after: ['repeat', 'repeat', 'caller'],
    retainsRows: true,
  },
  reordered: {
    name: 'reordered repeated frames',
    before: ['repeat', 'repeat', 'caller'],
    after: ['caller', 'repeat', 'repeat'],
  },
  fewer: {
    name: 'fewer repetitions',
    before: ['repeat', 'repeat', 'repeat', 'caller'],
    after: ['caller', 'repeat'],
  },
  more: {
    name: 'more repetitions',
    before: ['caller', 'repeat'],
    after: ['repeat', 'repeat', 'repeat', 'caller'],
  },
  callerOnly: {
    name: 'only the caller',
    before: ['repeat', 'repeat', 'caller'],
    after: ['caller'],
  },
  empty: {
    name: 'an empty stack',
    before: ['repeat', 'repeat', 'caller'],
    after: [],
  },
};

function createStackError(names: readonly FrameName[], format: StackFormat, unparsedLines: boolean): Error {
  const lines = names.map(name => frames[name][format]);
  return createError('Stack update', {
    stack: ['Error: Stack update', ...(unparsedLines ? ['unrecognized stack metadata'] : []), ...lines].join('\n'),
  });
}

function createFrameUpdateStory(update: FrameUpdate, format: StackFormat, unparsedLines: boolean): Story {
  return {
    name: `${format} ${update.name} (unparsed metadata ${unparsedLines})`,
    play: async ({mount, canvasElement}) => {
      const {before, after, retainsRows} = update;
      const reset = fn<() => void>();
      const initialError = createStackError(before, format, unparsedLines);
      const nextError = retainsRows ? initialError : createStackError(after, format, unparsedLines);
      const canvas = await mount(<ErrorTransition initialError={initialError} nextError={nextError} reset={reset} />);
      const rowNodes = () => Array.from(canvasElement.getElementsByClassName(styles.frameRow));
      const readRows = () => rowNodes().map(row => row.textContent);
      await expect(readRows()).toEqual(before.map(name => frames[name].text));
      await userEvent.click(await canvas.findByRole('button', {name: /Stack trace/}));
      const initialRows = rowNodes();
      await userEvent.click(await canvas.findByRole('button', {name: 'Update error'}));
      await waitFor(async () => {
        await expect(readRows()).toEqual(after.map(name => frames[name].text));
      });
      if (retainsRows) {
        await Promise.all(
          rowNodes().map(async (row, position) => {
            await expect(row).toBe(initialRows[position]);
          }),
        );
      }
      await expect(reset).not.toHaveBeenCalled();
    },
  };
}

// Each matrix row is a separate export so Storybook reports its failure independently.
export const V8UnchangedFrames = createFrameUpdateStory(frameUpdates.unchanged, 'v8', false);
export const V8ReorderedFrames = createFrameUpdateStory(frameUpdates.reordered, 'v8', false);
export const V8FewerFrames = createFrameUpdateStory(frameUpdates.fewer, 'v8', false);
export const V8MoreFrames = createFrameUpdateStory(frameUpdates.more, 'v8', false);
export const V8OnlyCaller = createFrameUpdateStory(frameUpdates.callerOnly, 'v8', false);
export const V8EmptyStack = createFrameUpdateStory(frameUpdates.empty, 'v8', false);
export const V8WithMetadataUnchangedFrames = createFrameUpdateStory(frameUpdates.unchanged, 'v8', true);
export const V8WithMetadataReorderedFrames = createFrameUpdateStory(frameUpdates.reordered, 'v8', true);
export const V8WithMetadataFewerFrames = createFrameUpdateStory(frameUpdates.fewer, 'v8', true);
export const V8WithMetadataMoreFrames = createFrameUpdateStory(frameUpdates.more, 'v8', true);
export const V8WithMetadataOnlyCaller = createFrameUpdateStory(frameUpdates.callerOnly, 'v8', true);
export const V8WithMetadataEmptyStack = createFrameUpdateStory(frameUpdates.empty, 'v8', true);
export const WebKitUnchangedFrames = createFrameUpdateStory(frameUpdates.unchanged, 'webkit', false);
export const WebKitReorderedFrames = createFrameUpdateStory(frameUpdates.reordered, 'webkit', false);
export const WebKitFewerFrames = createFrameUpdateStory(frameUpdates.fewer, 'webkit', false);
export const WebKitMoreFrames = createFrameUpdateStory(frameUpdates.more, 'webkit', false);
export const WebKitOnlyCaller = createFrameUpdateStory(frameUpdates.callerOnly, 'webkit', false);
export const WebKitEmptyStack = createFrameUpdateStory(frameUpdates.empty, 'webkit', false);
export const WebKitWithMetadataUnchangedFrames = createFrameUpdateStory(frameUpdates.unchanged, 'webkit', true);
export const WebKitWithMetadataReorderedFrames = createFrameUpdateStory(frameUpdates.reordered, 'webkit', true);
export const WebKitWithMetadataFewerFrames = createFrameUpdateStory(frameUpdates.fewer, 'webkit', true);
export const WebKitWithMetadataMoreFrames = createFrameUpdateStory(frameUpdates.more, 'webkit', true);
export const WebKitWithMetadataOnlyCaller = createFrameUpdateStory(frameUpdates.callerOnly, 'webkit', true);
export const WebKitWithMetadataEmptyStack = createFrameUpdateStory(frameUpdates.empty, 'webkit', true);
