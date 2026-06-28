import '@radix-ui/themes/styles.css';
import '#main.css';
import './radix-ui-theme.css';
import './surface.css';

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
  // The `themeSurface` global selects the production surface (see `withTheme.tsx`). Its toolbar UI
  // is a manager-side custom tool (see `.storybook/manager.tsx`) rather than a declarative
  // `globalTypes.toolbar`, so that "Reset" can truly clear the global (set it to `undefined`),
  // returning to no selection — a declarative item list can only ever set a value.
  globalTypes: {
    themeSurface: {
      name: 'Surface',
      description: 'Production surface the story renders inside',
    },
  },
  // Outer-to-inner wrap order: QueryClient -> Theme -> Story.
  decorators: [withQueryClient, withTheme],
};

export default preview;
