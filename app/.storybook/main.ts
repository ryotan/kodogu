import {dirname, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';

import {mergeConfig} from 'vite';

import type {StorybookConfig} from '@storybook/react-vite';

const __dirname = dirname(fileURLToPath(import.meta.url));

/**
 * This function is used to resolve the absolute path of a package.
 * It is needed in projects that use Yarn PnP or are set up within a monorepo.
 */
function getAbsolutePath(value: string) {
  return dirname(fileURLToPath(import.meta.resolve(`${value}/package.json`)));
}
const config: StorybookConfig = {
  stories: ['../src/**/*.mdx', '../src/**/*.stories.@(js|jsx|mjs|ts|tsx)'],
  addons: [
    getAbsolutePath('@chromatic-com/storybook'),
    getAbsolutePath('@storybook/addon-vitest'),
    getAbsolutePath('@storybook/addon-a11y'),
    getAbsolutePath('@storybook/addon-docs'),
    getAbsolutePath('@storybook/addon-onboarding'),
  ],
  framework: getAbsolutePath('@storybook/react-vite'),
  // preview.tsx lives under preview/ so the .storybook/ root only holds Node-side config.
  // Storybook auto-discovers `<config-dir>/preview.{ts,tsx}` by convention; previewAnnotations
  // points it at the relocated entry instead. Use an absolute path because Storybook resolves
  // these relative to the Vite project root (= app/), not to the config dir.
  previewAnnotations: [resolve(__dirname, 'preview/preview.tsx')],
  viteFinal: config =>
    mergeConfig(config, {
      // Storybook renders components in isolation; dev-only UI (devtools panels, stack-trace UI)
      // would only add noise to stories.
      define: {
        __DEV_TOOLS__: 'false',
      },
      resolve: {
        alias: {
          // Tauri IPC is not available in the browser — stub @tauri-apps/api/core.
          // The stub lives under app/src/(dev)/ so the same module can be reused outside Storybook
          // in the future (e.g. a browser-only vitest project).
          '@tauri-apps/api/core': resolve(__dirname, '../src/(dev)/tauri/core.stub.ts'),
        },
      },
    }),
};
export default config;
