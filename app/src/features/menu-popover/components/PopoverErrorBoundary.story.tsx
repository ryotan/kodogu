import {Text} from '@radix-ui/themes';

import type {Meta, StoryObj} from '@storybook/react-vite';
import {expect, userEvent} from 'storybook/test';

import {PopoverErrorBoundary} from './PopoverErrorBoundary';

const meta: Meta<typeof PopoverErrorBoundary> = {
  component: PopoverErrorBoundary,
  parameters: {theme: {surface: 'popover-dark'}},
};
export default meta;

type Story = StoryObj<typeof PopoverErrorBoundary>;

const AlwaysThrows = () => {
  throw new Error('boundary test error');
};

export const FallbackOnThrow: Story = {
  render: () => (
    <PopoverErrorBoundary>
      <AlwaysThrows />
    </PopoverErrorBoundary>
  ),
  play: async ({canvas}) => {
    await expect(canvas.getByRole('button', {name: 'もう一度試す'})).toBeVisible();
  },
};

// The retry button only resets the boundary and re-renders the children; the children recover only
// once the underlying condition has cleared. The play flips the flag before retrying to model
// "the failing dependency recovered, then the user retried". `shouldThrow` is module-scoped but only
// this story mutates it, and each story runs once per `vitest run`.
let shouldThrow = true;
function ConditionalThrow() {
  if (shouldThrow) {
    throw new Error('boundary test error');
  }
  return <Text>recovered</Text>;
}

export const RecoversAfterRetry: Story = {
  render: () => (
    <PopoverErrorBoundary>
      <ConditionalThrow />
    </PopoverErrorBoundary>
  ),
  play: async ({canvas}) => {
    await expect(canvas.getByRole('button', {name: 'もう一度試す'})).toBeVisible();
    shouldThrow = false;
    await userEvent.click(canvas.getByRole('button', {name: 'もう一度試す'}));
    await expect(canvas.getByText('recovered')).toBeVisible();
  },
};
