import type {Meta, StoryObj} from '@storybook/react-vite';

import {PopoverPendingLayout} from './PopoverPendingLayout';

const meta = {
  component: PopoverPendingLayout,
  parameters: {layout: 'fullscreen'},
} satisfies Meta<typeof PopoverPendingLayout>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
