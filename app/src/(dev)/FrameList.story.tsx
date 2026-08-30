import type {Meta, StoryObj} from '@storybook/react-vite';
import {expect} from 'storybook/test';

import {FrameList} from './FrameList';

const meta: Meta<typeof FrameList> = {
  component: FrameList,
};
export default meta;

type Story = StoryObj<typeof FrameList>;

export const Empty: Story = {
  args: {frames: [], emptyMessage: 'No frames available'},
  play: async ({canvas}) => {
    await expect(canvas.getByText('No frames available')).toBeVisible();
  },
};

export const SingleName: Story = {
  args: {frames: [{name: 'MyComponent'}], emptyMessage: 'No frames'},
  play: async ({canvas}) => {
    await expect(canvas.getByText('MyComponent')).toBeVisible();
  },
};

export const NameWithLocation: Story = {
  args: {
    frames: [{name: 'MyComponent', location: 'src/foo.tsx:42:18'}],
    emptyMessage: 'No frames',
  },
  play: async ({canvas}) => {
    await expect(canvas.getByText('MyComponent')).toBeVisible();
    await expect(canvas.getByText(/src\/foo\.tsx:42:18/)).toBeVisible();
  },
};

export const WithBadge: Story = {
  args: {
    frames: [{badge: {text: 'app', color: 'blue'}, name: 'MyComponent'}],
    emptyMessage: 'No frames',
  },
  play: async ({canvas}) => {
    await expect(canvas.getByText('app')).toBeVisible();
    await expect(canvas.getByText('MyComponent')).toBeVisible();
  },
};

export const Multiple: Story = {
  args: {
    frames: [{name: 'First'}, {name: 'Second'}, {name: 'Third'}],
    emptyMessage: 'No frames',
  },
  play: async ({canvas}) => {
    await expect(canvas.getByText('First')).toBeVisible();
    await expect(canvas.getByText('Second')).toBeVisible();
    await expect(canvas.getByText('Third')).toBeVisible();
  },
};

export const ExternalIsDimmed: Story = {
  args: {
    frames: [
      {name: 'loadConnections', location: 'src/adapters.ts:42', isExternal: false},
      {name: 'renderWithHooks', location: 'node_modules/.vite/deps/react-dom.js:8231', isExternal: true},
    ],
    emptyMessage: 'No frames',
  },
  play: async ({canvas}) => {
    await expect(canvas.getByText('loadConnections')).toBeVisible();
    await expect(canvas.getByText('renderWithHooks')).toBeVisible();
  },
};
