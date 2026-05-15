// Story for the AppLayout component. Storybook runs with __DEV_TOOLS__ = false, so the rendered
// output omits the Radix theme panel and just shows the Theme-wrapped Outlet content.

import type {Meta, StoryObj} from '@storybook/react-vite';

import {AppLayout} from '#components/AppLayout';
import {withMemoryRouter} from '$preview/withMemoryRouter';

const meta: Meta<typeof AppLayout> = {
  component: AppLayout,
  decorators: [withMemoryRouter],
};
export default meta;

type Story = StoryObj<typeof AppLayout>;

export const Default: Story = {};
