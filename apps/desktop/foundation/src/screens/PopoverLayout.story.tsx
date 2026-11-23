import type {Meta, StoryObj} from '@storybook/react-vite';
import {expect, mocked, waitFor, within} from 'storybook/test';

import {resizePopover} from '#commands/popover';

import {Route} from './popover';

const PopoverLayout = Route.options.component;
if (!PopoverLayout) {
  throw new Error('PopoverLayout component is not defined');
}

const meta = {
  component: PopoverLayout,
  parameters: {layout: 'fullscreen'},
} satisfies Meta<typeof PopoverLayout>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Sections: Story = {
  play: async ({canvasElement}) => {
    const canvas = within(canvasElement);
    await expect(await canvas.findByText('2接続中 / 4プロファイル')).toBeVisible();
    await expect(await canvas.findByText('接続中', {exact: true})).toBeVisible();
  },
};

export const ResizesPopover: Story = {
  beforeEach: () => {
    mocked(resizePopover).mockClear();
  },
  play: async () => {
    await waitFor(async () => {
      await expect(resizePopover).toHaveBeenCalledWith(expect.any(Number), expect.any(Number));
    });
  },
};
