import '@radix-ui/themes/styles.css';
import '../src/main.css';
import './radix-ui-theme.css';

import type {FC, ReactNode} from 'react';

import {QueryClientProvider} from '@tanstack/react-query';

import {Theme} from '@radix-ui/themes';

import type {Preview} from '@storybook/react-vite';

import {createQueryClient} from '#createQueryClient';
import {useStable} from '#hooks/useStable';

// A client per story keeps one story's cached data and errors out of the others.
const StoryQueryClientProvider = ({children}: {children: ReactNode}) => {
  const queryClient = useStable(createQueryClient);
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
};

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
        <StoryQueryClientProvider>
          <Theme panelBackground='solid'>
            <Story />
          </Theme>
        </StoryQueryClientProvider>
      );
    },
  ],
};

export default preview;
