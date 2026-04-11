// app/src/shared/core/errors/ErrorDisplay.stories.tsx
import type {Meta, StoryObj} from '@storybook/react';

import {Theme} from '@radix-ui/themes';

import {ApplicationError} from './ApplicationError';
import {TypedAggregateError} from './TypedAggregateError';
import {ErrorDisplay} from './ErrorDisplay';

const meta: Meta<typeof ErrorDisplay> = {
  component: ErrorDisplay,
  decorators: [
    (Story) => (
      <Theme appearance="dark" hasBackground={false}>
        <Story />
      </Theme>
    ),
  ],
};
export default meta;

type Story = StoryObj<typeof ErrorDisplay>;

export const BasicError: Story = {
  args: {
    error: Object.assign(new Error('Something went wrong'), {
      stack: 'Error: Something went wrong',
    }),
  },
};

export const WithOriginFrame: Story = {
  args: {
    error: Object.assign(new ApplicationError('Config load failed'), {
      stack: [
        'ApplicationError: Config load failed',
        'getConfig@http://localhost:5173/src/hooks/useAppConfig.ts:12:5',
        'renderWithHooks@http://localhost:5173/node_modules/.vite/deps/react-dom_client.js:100:1',
      ].join('\n'),
    }),
  },
};

export const WithAllExternalFrames: Story = {
  args: {
    error: Object.assign(new Error('External only'), {
      stack: [
        'Error: External only',
        'chunk@http://localhost:5173/node_modules/.vite/deps/react.js:1:1',
        'renderWithHooks@http://localhost:5173/node_modules/.vite/deps/react-dom_client.js:100:1',
      ].join('\n'),
    }),
  },
};

export const WithCausedBy: Story = {
  args: {
    error: Object.assign(new ApplicationError('Outer error', {cause: new Error('Inner cause')}), {
      stack: 'ApplicationError: Outer error\nfoo@http://localhost:5173/src/foo.ts:1:1',
    }),
  },
};

export const WithAggregateErrors: Story = {
  args: {
    error: Object.assign(
      new ApplicationError('Validation failed', {
        cause: new TypedAggregateError(
          [
            Object.assign(new ApplicationError('name is required'), {stack: 'ApplicationError: name is required'}),
            Object.assign(new ApplicationError('port out of range'), {stack: 'ApplicationError: port out of range'}),
          ],
          '2 validation errors',
        ),
      }),
      {stack: 'ApplicationError: Validation failed\nvalidate@http://localhost:5173/src/foo.ts:10:5'},
    ),
  },
};
