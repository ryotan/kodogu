// Story for the application's root route layout. Storybook runs with __DEV_TOOLS__ = false,
// so the rendered output omits TanStack devtools and just shows the routed Outlet content.

import type {Meta, StoryObj} from '@storybook/react-vite';

import {withMemoryRouter} from '$preview/withMemoryRouter';

import {RootLayout} from './__root';

const meta: Meta<typeof RootLayout> = {
  component: RootLayout,
  decorators: [withMemoryRouter],
};
export default meta;

type Story = StoryObj<typeof RootLayout>;

export const Default: Story = {};
