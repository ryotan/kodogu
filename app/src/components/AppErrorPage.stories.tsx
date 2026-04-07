import type {Meta, StoryObj} from '@storybook/react-vite';
import {fn} from 'storybook/test';

import {AppErrorPage} from './AppErrorPage';

const meta = {
  component: AppErrorPage,
  parameters: {layout: 'fullscreen'},
  args: {
    error: new Error('Something went wrong'),
    reset: fn(),
  },
} satisfies Meta<typeof AppErrorPage>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const WithStackTrace: Story = {
  args: {
    error: (() => {
      const e = new Error('Failed to load config');
      e.stack = `Error: Failed to load config
    at useAppConfig (http://localhost:5173/src/hooks/useAppConfig.ts:7:14)
    at renderWithHooks (http://localhost:5173/node_modules/react-dom/client.js:123:10)`;
      return e;
    })(),
  },
};
