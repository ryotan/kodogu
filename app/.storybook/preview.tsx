import '@radix-ui/themes/styles.css';
import '../src/main.css';
import './radix-ui-theme.css';

import type {FC} from 'react';

import {QueryClient, QueryClientProvider, QueryErrorResetBoundary} from '@tanstack/react-query';

import {Theme} from '@radix-ui/themes';

import type {Preview} from '@storybook/react-vite';

import {defaultAppConfig} from '../src/commands/appConfig.fixture';
import {appConfigQuery} from '../src/hooks/useAppConfig';

const queryClient = new QueryClient();
// Pre-populate the AppConfig cache so route layouts that read it via useSuspenseQuery
// (e.g. RootLayout's useConfiguredDesignTokens hook) render synchronously in stories.
queryClient.setQueryData(appConfigQuery.queryKey, defaultAppConfig);

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
