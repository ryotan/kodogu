import '@radix-ui/themes/styles.css';
import '#main.css';
import './radix-ui-theme.css';

import type {Preview} from '@storybook/react-vite';

import {withQueryClient} from './withQueryClient';
import {withTheme} from './withTheme';

const preview: Preview = {
  parameters: {
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
    },
  },
  // Outer-to-inner wrap order: QueryClient -> Theme -> Story.
  decorators: [withQueryClient, withTheme],
};

export default preview;
