import {Card, Inset} from '@radix-ui/themes';

import type {Meta, StoryObj} from '@storybook/react-vite';
import {expect, fn, userEvent} from 'storybook/test';

import {ApplicationError} from './ApplicationError';
import {ErrorPage} from './ErrorPage';
import {TypedAggregateError} from './TypedAggregateError';

const meta: Meta<typeof ErrorPage> = {
  component: ErrorPage,
  // ErrorPage is framing-agnostic; stories show it inside the recommended `<Card><Inset side='all'>`.
  decorators: [
    Story => (
      <Card size='3'>
        <Inset side='all'>
          <Story />
        </Inset>
      </Card>
    ),
  ],
  parameters: {surface: {value: 'popover-dark'}},
};
export default meta;

type Story = StoryObj<typeof ErrorPage>;

export const Simple: Story = {
  args: {error: new ApplicationError('接続情報の読み込みに失敗しました'), reset: fn()},
  play: async ({canvas, args}) => {
    await expect(canvas.getByText('接続情報の読み込みに失敗しました')).toBeVisible();
    const button = canvas.getByRole('button', {name: 'もう一度試す'});
    await expect(button).toBeVisible();
    await userEvent.click(button);
    await expect(args.reset).toHaveBeenCalledOnce();
  },
};

export const Aggregate: Story = {
  args: {
    error: new ApplicationError('入力に不備があります', {
      cause: new TypedAggregateError(
        [new ApplicationError('名前は必須です'), new ApplicationError('ポート番号は1〜65535の範囲で入力してください')],
        '2件の検証エラー',
      ),
    }),
    reset: fn(),
  },
  play: async ({canvas}) => {
    await expect(canvas.getByText('入力に不備があります')).toBeVisible();
    await expect(canvas.getByText('名前は必須です')).toBeVisible();
    await expect(canvas.getByText('ポート番号は1〜65535の範囲で入力してください')).toBeVisible();
  },
};

export const NoRetryWhenResetOmitted: Story = {
  args: {error: new ApplicationError('処理に失敗しました')},
  play: async ({canvas}) => {
    await expect(canvas.getByText('処理に失敗しました')).toBeVisible();
    // No `reset` prop → no retry button is rendered. The error surface is
    // purely informational in this mode.
    await expect(canvas.queryByRole('button', {name: 'もう一度試す'})).toBeNull();
  },
};

export const Nested: Story = {
  args: {
    error: new ApplicationError('設定の保存に失敗しました', {
      cause: new ApplicationError('storage write rejected', {
        cause: new ApplicationError('permission denied'),
      }),
    }),
    errorInfo: {
      componentStack:
        '\n    at SettingsForm (src/features/settings/SettingsForm.tsx:42:18)' +
        '\n    at Suspense\n    at RouterProvider',
    },
    reset: fn(),
  },
  play: async ({canvas}) => {
    // Only the top-level message is user-facing (Pattern A). The cause chain
    // lives in the dev-only Debug info section and is not asserted here.
    await expect(canvas.getByText('設定の保存に失敗しました')).toBeVisible();
    await expect(canvas.queryByText('storage write rejected')).toBeNull();
  },
};
