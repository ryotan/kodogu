// Story for the AppLayout component. Storybook runs with __DEV_TOOLS__ = false, so the rendered
// output omits the Radix theme panel and just shows the Theme-wrapped Outlet content.

import type {Meta, StoryObj} from '@storybook/react-vite';

import {AppLayout} from '#components/AppLayout';
import {renderInRouter} from '$storybook/helpers/createStoryRouter';

const AppLayoutStory = () => renderInRouter(AppLayout);

const meta: Meta<typeof AppLayoutStory> = {
  component: AppLayoutStory,
};
export default meta;

type Story = StoryObj<typeof AppLayoutStory>;

export const Default: Story = {};
