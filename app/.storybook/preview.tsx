import '@radix-ui/themes/styles.css';
import '../src/main.css';
import './radix-ui-theme.css';

import type {FC} from 'react';

import {Theme} from '@radix-ui/themes';

import type {Preview} from '@storybook/react-vite';

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
        <Theme panelBackground='solid'>
          <Story />
        </Theme>
      );
    },
  ],
};

export default preview;
