import '@radix-ui/themes/styles.css';
import '#main.css';
import './surface.css';

import type {Preview} from '@storybook/react-vite';

import {SURFACE_GLOBAL_KEY} from './surfaces';
import {withQueryClient} from './withQueryClient';
import {withSurface} from './withSurface.tsx';

const preview: Preview = {
  parameters: {
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
    },
  },
  // The `surface` global selects the production surface (see `withSurface.tsx`). Its toolbar UI
  // is a manager-side custom tool (see `.storybook/manager.tsx`) rather than a declarative
  // `globalTypes.toolbar`, so that "Reset" can truly clear the global (set it to `undefined`),
  // returning to no selection — a declarative item list can only ever set a value.
  globalTypes: {
    [SURFACE_GLOBAL_KEY]: {
      name: 'Surface',
      description: 'Production surface the story renders inside',
    },
  },
  // Outer-to-inner wrap order: QueryClient -> Theme -> Story.
  decorators: [withQueryClient, withSurface],
};

export default preview;
