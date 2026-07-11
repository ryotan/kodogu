import {QueryErrorResetBoundary} from '@tanstack/react-query';

import type {Meta, StoryObj} from '@storybook/react-vite';
import {expect, fn, userEvent} from 'storybook/test';

import {ApplicationError} from '#shared/core/errors/ApplicationError';
import {TypedAggregateError} from '#shared/core/errors/TypedAggregateError';

import {PopoverAppErrorPage} from './PopoverAppErrorPage';

const meta: Meta<typeof PopoverAppErrorPage> = {
  component: PopoverAppErrorPage,
  // The component reads `useQueryErrorResetBoundary` to clear failed queries
  // on retry; wrap each story in the boundary so the hook resolves.
  decorators: [
    Story => (
      <QueryErrorResetBoundary>
        <Story />
      </QueryErrorResetBoundary>
    ),
  ],
  parameters: {surface: {value: 'popover-dark'}},
};
export default meta;

type Story = StoryObj<typeof PopoverAppErrorPage>;

// The RetryAlsoResetsQueries story stashes the query-reset boundary on `window` so its play function
// can assert the reset side effect. Typed here (rather than `any`) to keep the escape hatch narrow.
type QueryBoundaryWindow = typeof globalThis & {__queryBoundary?: {isReset: () => boolean}};

const sampleAggregateError = new ApplicationError('入力に不備があります', {
  cause: new TypedAggregateError(
    [new ApplicationError('名前は必須です'), new ApplicationError('ポート番号は1〜65535の範囲で入力してください')],
    '2件の検証エラー',
  ),
});

export const SimpleError: Story = {
  args: {
    error: new ApplicationError('接続情報の読み込みに失敗しました'),
    reset: fn(),
  },
  play: async ({canvas, args}) => {
    await expect(canvas.getByText('接続情報の読み込みに失敗しました')).toBeVisible();
    await userEvent.click(canvas.getByRole('button', {name: 'もう一度試す'}));
    await expect(args.reset).toHaveBeenCalledOnce();
  },
};

export const Aggregate: Story = {
  args: {
    error: sampleAggregateError,
    reset: fn(),
  },
  play: async ({canvas}) => {
    await expect(canvas.getByText('入力に不備があります')).toBeVisible();
    await expect(canvas.getByText('名前は必須です')).toBeVisible();
    await expect(canvas.getByText('ポート番号は1〜65535の範囲で入力してください')).toBeVisible();
  },
};

export const RetryAlsoResetsQueries: Story = {
  args: {
    error: new ApplicationError('一時的なエラーが発生しました'),
    reset: fn(),
  },
  // QueryErrorResetBoundary.reset() marks the boundary as reset (isReset = true),
  // letting suspended queries re-throw and retry on the next render. We confirm
  // the side effect via a render-callback boundary instead of an opaque mock.
  decorators: [
    Story => (
      <QueryErrorResetBoundary>
        {value => {
          (window as QueryBoundaryWindow).__queryBoundary = value;
          return <Story />;
        }}
      </QueryErrorResetBoundary>
    ),
  ],
  play: async ({canvas, args}) => {
    const boundary = (window as QueryBoundaryWindow).__queryBoundary;
    if (!boundary) {
      throw new Error('query boundary was not stashed by the decorator');
    }
    await expect(boundary.isReset()).toBe(false);
    await userEvent.click(canvas.getByRole('button', {name: 'もう一度試す'}));
    await expect(args.reset).toHaveBeenCalledOnce();
    await expect(boundary.isReset()).toBe(true);
  },
};
