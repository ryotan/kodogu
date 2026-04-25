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
  viteFinal: config =>
    mergeConfig(config, {
      // Storybook renders components in isolation; dev-only UI (devtools panels, stack-trace UI)
      // would only add noise to stories.
      define: {
        __DEV_TOOLS__: 'false',
      },
      resolve: {
        alias: {
          // Tauri IPC is not available in the browser — stub @tauri-apps/api/core
          '@tauri-apps/api/core': resolve(__dirname, 'tauri-api-stub.ts'),
        },
      },
    }),
};
export default config;
