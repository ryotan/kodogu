import '@radix-ui/themes/styles.css';
import '../src/main.css';
import './radix-ui-theme.css';

import type {FC} from 'react';

import {QueryClient, QueryClientProvider, QueryErrorResetBoundary} from '@tanstack/react-query';

import {Theme} from '@radix-ui/themes';

import type {Preview} from '@storybook/react-vite';

const queryClient = new QueryClient();

const preview: Preview = {
  parameters: {
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
    },
  },
  decorators: [
    (Story: FC) => {
      return (
        <QueryClientProvider client={queryClient}>
          <QueryErrorResetBoundary>
            <Theme panelBackground='solid'>
              <Story />
            </Theme>
          </QueryErrorResetBoundary>
        </QueryClientProvider>
      );
    },
  ],
};

export default preview;
