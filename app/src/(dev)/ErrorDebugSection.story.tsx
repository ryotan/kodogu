import {Box} from '@radix-ui/themes';

import type {Meta, StoryObj} from '@storybook/react-vite';
import {expect, userEvent} from 'storybook/test';

import {ApplicationError} from '#shared/core/errors/ApplicationError';
import {PlatformError} from '#shared/core/errors/PlatformError';
import {TypedAggregateError} from '#shared/core/errors/TypedAggregateError';

import {ErrorDebugSection} from './ErrorDebugSection';

// ErrorDebugSection doesn't read __DEV_TOOLS__ (only ErrorPage's lazy import does), so it renders in
// the default storybook build. The build-flag gating itself is covered separately by
// devGuards.devtools.spec; these stories cover what the section *renders* — the cause tree, raw and
// cutoff leaves, the external-frame switch, and the component stack.
const meta: Meta<typeof ErrorDebugSection> = {
  component: ErrorDebugSection,
  parameters: {theme: {surface: 'app-dark', fill: false}},
  decorators: [
    Story => (
      <Box width='420px'>
        <Story />
      </Box>
    ),
  ],
};
export default meta;

type Story = StoryObj<typeof ErrorDebugSection>;

const openDebugInfo = (canvas: Parameters<NonNullable<Story['play']>>[0]['canvas']) =>
  userEvent.click(canvas.getByRole('button', {name: /debug info/i}));

// A node's causes are always expanded, so opening Debug info reveals the whole chain; only frames and
// the component stack stay collapsed.
export const NestedCauses: Story = {
  args: {
    error: new ApplicationError('設定の保存に失敗しました', {
      cause: new ApplicationError('storage write rejected', {
        cause: new ApplicationError('permission denied'),
      }),
    }),
  },
  play: async ({canvas}) => {
    await openDebugInfo(canvas);
    await expect(canvas.getByText(/storage write rejected/)).toBeVisible();
    await expect(canvas.getByText(/permission denied/)).toBeVisible();
  },
};

// An aggregate cause renders as its own node (not unwrapped) with its members as sibling children.
export const Aggregate: Story = {
  args: {
    error: new ApplicationError('入力に不備があります', {
      cause: new TypedAggregateError(
        [new ApplicationError('名前は必須です'), new ApplicationError('ポート番号は1〜65535の範囲で入力してください')],
        '2件の検証エラー',
      ),
    }),
  },
  play: async ({canvas}) => {
    await openDebugInfo(canvas);
    await expect(canvas.getByText(/TypedAggregateError/)).toBeVisible();
    await expect(canvas.getByText(/名前は必須です/)).toBeVisible();
    await expect(canvas.getByText(/ポート番号は1〜65535の範囲で入力してください/)).toBeVisible();
  },
};

// A non-Error cause is kept (debug fidelity) as a raw leaf rather than dropped.
export const RawCause: Story = {
  args: {
    error: new ApplicationError('外部コマンドが失敗しました', {cause: 'exited with signal SIGKILL'}),
  },
  play: async ({canvas}) => {
    await openDebugInfo(canvas);
    await expect(canvas.getByText(/exited with signal SIGKILL/)).toBeVisible();
  },
};

const makeCyclicError = () => {
  const a = new ApplicationError('A');
  const b = new ApplicationError('B');
  a.cause = b;
  b.cause = a;
  return a;
};

// A cycle is cut at the re-entry point and shown as a leaf, never silently dropped or looped.
// Built inside `render`, not `args`: a circular object breaks Storybook's args serialization
// (controls/docs), which would blank the story canvas even though buildErrorTree handles the cycle.
export const Cycle: Story = {
  render: () => <ErrorDebugSection error={makeCyclicError()} />,
  play: async ({canvas}) => {
    await openDebugInfo(canvas);
    await expect(canvas.getByText(/cycle/)).toBeVisible();
  },
};

// A synthetic stack mixing app and node_modules frames, to show external frames being dimmed and
// the section-level Show all / Internal only switch revealing them.
const mixedFramesError = (() => {
  const error = new ApplicationError('フォームの送信に失敗しました');
  error.stack = [
    'ApplicationError: フォームの送信に失敗しました',
    '    at submitSettings (http://localhost:6006/src/features/settings/useSettingsForm.ts:20:9)',
    '    at renderWithHooks (http://localhost:6006/node_modules/.vite/deps/react-dom_client.js:11548:26)',
    '    at updateFunctionComponent (http://localhost:6006/node_modules/.vite/deps/react-dom_client.js:14926:21)',
    '    at SettingsForm (http://localhost:6006/src/features/settings/SettingsForm.tsx:42:18)',
  ].join('\n');
  return error;
})();

export const ExternalFrames: Story = {
  args: {error: mixedFramesError},
  play: async ({canvas}) => {
    await openDebugInfo(canvas);
    // The header row is the toggle; expand this node's frames, then reveal the external ones.
    await userEvent.click(canvas.getByRole('button', {name: /フォームの送信に失敗しました/}));
    await userEvent.click(canvas.getByRole('button', {name: /show all/i}));
    await expect(canvas.getAllByText('ext')[0]).toBeVisible();
  },
};

// Actually thrown (not fabricated): a genuine Error object with a real stack and a real cause chain
// (a PlatformError wrapped by an ApplicationError), to verify the view against a real error.
const thrownError = (() => {
  const callIpc = () => {
    throw new PlatformError('IPC コマンドの呼び出しに失敗しました');
  };
  const loadConnections = () => {
    try {
      callIpc();
    } catch (cause) {
      throw new ApplicationError('接続情報の読み込みに失敗しました', {cause: cause as Error});
    }
  };
  try {
    loadConnections();
  } catch (error) {
    return error as Error;
  }
  throw new Error('unreachable');
})();

export const RealError: Story = {
  args: {error: thrownError},
  play: async ({canvas}) => {
    await openDebugInfo(canvas);
    await expect(canvas.getByText(/接続情報の読み込みに失敗しました/)).toBeVisible();
    await expect(canvas.getByText(/IPC コマンドの呼び出しに失敗しました/)).toBeVisible();
  },
};

// The component stack is a separate labelled disclosure, shown only when errorInfo carries one.
export const WithComponentStack: Story = {
  args: {
    error: new ApplicationError('設定の保存に失敗しました'),
    errorInfo: {
      componentStack:
        '\n    at SettingsForm (src/features/settings/SettingsForm.tsx:42:18)\n    at Suspense\n    at RouterProvider',
    },
  },
  play: async ({canvas}) => {
    await openDebugInfo(canvas);
    await userEvent.click(canvas.getByRole('button', {name: /component stack/i}));
    await expect(canvas.getByText(/SettingsForm\.tsx/)).toBeVisible();
  },
};
