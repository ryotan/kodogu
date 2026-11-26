import '@radix-ui/themes/styles.css';
import '../main.css';
import './radix-ui-theme.css';

import type {FC, ReactNode} from 'react';

import {QueryClientProvider} from '@tanstack/react-query';

import {Theme} from '@radix-ui/themes';

import type {Preview} from '@storybook/react-vite';

import {createQueryClient} from '#createQueryClient';
import {useStable} from '#hooks/useStable';

// The decorators and parameters of the stories of the desktop app. The `.storybook/preview.tsx` of `foundation` and of
// each tool spreads them and adds its own mocks.

// A client per story keeps one story's cached data and errors out of the others.
function StoryQueryClientProvider({children}: {children: ReactNode}) {
  const queryClient = useStable(createQueryClient);
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
}

export const preview: Preview = {
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
