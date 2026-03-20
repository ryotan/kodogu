import {QueryClient, QueryClientProvider, QueryErrorResetBoundary} from '@tanstack/react-query';

import type {Meta, StoryObj} from '@storybook/react-vite';
import {expect, fn, userEvent, within} from 'storybook/test';

import {AppErrorComponent} from './AppErrorComponent';

function createError(message: string, options?: {name?: string; stack?: string}): Error {
  const error = new Error(message);
  if (options?.name) {
    error.name = options.name;
  }
  if (options?.stack) {
    error.stack = options.stack;
  }
  return error;
}

const queryClient = new QueryClient();

const meta = {
  component: AppErrorComponent,
  parameters: {layout: 'fullscreen'},
  decorators: [
    Story => (
      <QueryClientProvider client={queryClient}>
        <QueryErrorResetBoundary>
          <Story />
        </QueryErrorResetBoundary>
      </QueryClientProvider>
    ),
  ],
  args: {
    error: new Error('Something went wrong'),
    reset: fn(),
  },
} satisfies Meta<typeof AppErrorComponent>;

export default meta;
type Story = StoryObj<typeof meta>;

// -- Stack trace fixtures --

const v8Stack = `Error: Failed to fetch config
    at useAppConfig (http://localhost:5173/src/hooks/useAppConfig.ts:7:14)
    at renderWithHooks (http://localhost:5173/node_modules/react-dom/client.js:123:10)
    at mountIndeterminateComponent (http://localhost:5173/node_modules/react-dom/client.js:456:7)
    at beginWork (http://localhost:5173/node_modules/react-dom/client.js:789:3)`;

const webkitStack = `TypeError: Cannot read properties of undefined
renderWithHooks@http://localhost:5173/node_modules/react-dom/client.js:123:10
mountIndeterminateComponent@http://localhost:5173/node_modules/react-dom/client.js:456:7
beginWork@http://localhost:5173/.vite/deps/react-dom.js?v=abc:789:3`;

const mixedStack = `Error: Something broke
    at PopoverLayout (http://localhost:5173/src/routes/popover.tsx:22:5)
    at useAutoResize (http://localhost:5173/src/hooks/useAutoResize.ts:45:12)
    at renderWithHooks (http://localhost:5173/node_modules/react-dom/client.js:123:10)
    at beginWork (http://localhost:5173/node_modules/react-dom/client.js:789:3)`;

// -- Stories --

export const BasicError: Story = {};

export const WithStackTrace: Story = {
  args: {
    error: createError('Failed to fetch config', {stack: v8Stack}),
  },
};

export const AllExternalFrames: Story = {
  args: {
    error: createError('Cannot read properties of undefined', {name: 'TypeError', stack: webkitStack}),
  },
};

export const MixedFrames: Story = {
  args: {
    error: createError('Something broke', {stack: mixedStack}),
  },
};

export const LongErrorMessage: Story = {
  args: {
    error: createError(
      'Failed to connect to the backend service at https://api.example.com/v1/config. ' +
        'The server might be down or the network connection is unstable. ' +
        'Please check your internet connection and try again. ' +
        'If the problem persists, contact support with error code ERR_CONNECTION_REFUSED.',
      {name: 'NetworkError'},
    ),
  },
};

export const ExpandCollapseStackTrace: Story = {
  args: {
    error: createError('Something broke', {stack: mixedStack}),
  },
  play: async ({canvasElement}) => {
    const canvas = within(canvasElement);

    const expandButton = canvas.getByRole('button', {name: /stack trace/i});

    // Stack trace should be collapsed by default — grid-template-rows: 0fr
    const stackBody = expandButton.nextElementSibling;
    await expect(stackBody).not.toHaveAttribute('data-expanded');

    // Expand
    await userEvent.click(expandButton);
    await expect(stackBody).toHaveAttribute('data-expanded');

    // Verify a user-code frame is rendered
    await expect(canvas.getByText('PopoverLayout')).toBeInTheDocument();

    // Collapse
    await userEvent.click(expandButton);
    await expect(stackBody).not.toHaveAttribute('data-expanded');
  },
};
