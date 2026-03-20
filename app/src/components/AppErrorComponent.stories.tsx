import {QueryClient, QueryClientProvider, QueryErrorResetBoundary} from '@tanstack/react-query';

import type {Meta, StoryObj} from '@storybook/react-vite';
import {expect, userEvent, within} from 'storybook/test';

import {AppErrorComponent} from './AppErrorComponent';

function createError(name: string, message: string, stack?: string): Error {
  const error = new Error(message);
  error.name = name;
  error.stack = stack;
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
    reset: () => {
      // noop for storybook
    },
  },
} satisfies Meta<typeof AppErrorComponent>;

export default meta;
type Story = StoryObj<typeof meta>;

// -- Stack trace fixtures --

const v8StackWithInternalFrame = `Error: Failed to fetch config
    at useAppConfig (http://localhost:5173/src/hooks/useAppConfig.ts:7:14)
    at renderWithHooks (http://localhost:5173/node_modules/react-dom/client.js:123:10)
    at mountIndeterminateComponent (http://localhost:5173/node_modules/react-dom/client.js:456:7)
    at beginWork (http://localhost:5173/node_modules/react-dom/client.js:789:3)`;

const webkitStackAllExternal = `TypeError: Cannot read properties of undefined
renderWithHooks@http://localhost:5173/node_modules/react-dom/client.js:123:10
mountIndeterminateComponent@http://localhost:5173/node_modules/react-dom/client.js:456:7
beginWork@http://localhost:5173/.vite/deps/react-dom.js?v=abc:789:3`;

const mixedStack = `Error: Something broke
    at PopoverLayout (http://localhost:5173/src/routes/popover.tsx:22:5)
    at useAutoResize (http://localhost:5173/src/hooks/useAutoResize.ts:45:12)
    at renderWithHooks (http://localhost:5173/node_modules/react-dom/client.js:123:10)
    at beginWork (http://localhost:5173/node_modules/react-dom/client.js:789:3)`;

// -- Stories --

export const BasicError: Story = {
  args: {
    error: createError('Error', 'Something went wrong'),
  },
};

export const WithStackTraceAndOrigin: Story = {
  args: {
    error: createError('Error', 'Failed to fetch config', v8StackWithInternalFrame),
  },
};

export const AllExternalFrames: Story = {
  args: {
    error: createError('TypeError', 'Cannot read properties of undefined', webkitStackAllExternal),
  },
};

export const MixedInternalExternalFrames: Story = {
  args: {
    error: createError('Error', 'Something broke', mixedStack),
  },
};

export const LongErrorMessage: Story = {
  args: {
    error: createError(
      'NetworkError',
      'Failed to connect to the backend service at https://api.example.com/v1/config. ' +
        'The server might be down or the network connection is unstable. ' +
        'Please check your internet connection and try again. ' +
        'If the problem persists, contact support with error code ERR_CONNECTION_REFUSED.',
    ),
  },
};

export const ExpandCollapseStackTrace: Story = {
  args: {
    error: createError('Error', 'Something broke', mixedStack),
  },
  play: async ({canvasElement}) => {
    const canvas = within(canvasElement);

    // Stack trace should be collapsed by default
    const expandButton = canvas.getByRole('button', {name: /stack trace/i});
    await expect(expandButton).toBeInTheDocument();

    // Expand stack trace
    await userEvent.click(expandButton);

    // Frames should be visible after expanding
    const frames = canvas.getAllByText(/@|at/);
    await expect(frames.length).toBeGreaterThan(0);

    // Collapse again
    await userEvent.click(expandButton);
  },
};
