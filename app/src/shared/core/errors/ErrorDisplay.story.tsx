import type {Meta, StoryObj} from '@storybook/react-vite';
import {expect} from 'storybook/test';

import {ApplicationError} from './ApplicationError';
import {ErrorDisplay} from './ErrorDisplay';
import {PlatformError} from './PlatformError';
import {TypedAggregateError} from './TypedAggregateError';

const meta: Meta<typeof ErrorDisplay> = {
  component: ErrorDisplay,
};
export default meta;

type Story = StoryObj<typeof ErrorDisplay>;

export const ApplicationErrorMessage: Story = {
  args: {error: new ApplicationError('接続情報の読み込みに失敗しました')},
  play: async ({canvas}) => {
    await expect(canvas.getByText('接続情報の読み込みに失敗しました')).toBeVisible();
    // Pattern A: no AggregateError children → fallback subtitle is shown.
    await expect(canvas.getByText('時間をおいて、もう一度お試しください。')).toBeVisible();
  },
};

export const PlatformErrorMessage: Story = {
  args: {error: new PlatformError('設定の保存に失敗しました')},
  play: async ({canvas}) => {
    await expect(canvas.getByText('設定の保存に失敗しました')).toBeVisible();
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
  },
  play: async ({canvas}) => {
    await expect(canvas.getByText('入力に不備があります')).toBeVisible();
    await expect(canvas.getByText('名前は必須です')).toBeVisible();
    await expect(canvas.getByText('ポート番号は1〜65535の範囲で入力してください')).toBeVisible();
    // No fallback subtitle when aggregate children are present.
    await expect(canvas.queryByText('時間をおいて、もう一度お試しください。')).toBeNull();
  },
};

export const AggregateSkipsNonErrorItems: Story = {
  args: {
    error: new ApplicationError('入力に不備があります', {
      cause: new TypedAggregateError(
        // The `errors` array is typed as `Error[]` but a runtime may slip in
        // non-Error throws — we filter those out so arbitrary strings don't
        // leak into the UI.
        [new ApplicationError('名前は必須です'), 'a string was thrown' as unknown as Error],
        'mixed',
      ),
    }),
  },
  play: async ({canvas}) => {
    await expect(canvas.getByText('名前は必須です')).toBeVisible();
    await expect(canvas.queryByText('a string was thrown')).toBeNull();
  },
};

export const NonRootErrorFallsBack: Story = {
  args: {error: new Error('something English broke')},
  play: async ({canvas}) => {
    // Pattern A: errors outside the RootError hierarchy fall back to a
    // generic Japanese message — we don't want native English errors leaking
    // through.
    await expect(canvas.getByText('予期しないエラーが発生しました')).toBeVisible();
    await expect(canvas.queryByText('something English broke')).toBeNull();
  },
};
