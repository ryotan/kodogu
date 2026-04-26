// Story for the application's root route layout. Storybook runs with __DEV_TOOLS__ = false,
// so the rendered output omits TanStack devtools and just shows the routed Outlet content.

import type {Meta, StoryObj} from '@storybook/react-vite';

import {renderInRouter} from '$storybook/helpers/createStoryRouter';

import {RootLayout} from './__root';

const RootLayoutStory = () => renderInRouter(RootLayout);

const meta: Meta<typeof RootLayoutStory> = {
  component: RootLayoutStory,
};
export default meta;

type Story = StoryObj<typeof RootLayoutStory>;

export const Default: Story = {};
