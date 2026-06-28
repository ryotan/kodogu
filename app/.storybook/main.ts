import {dirname, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';

import {mergeConfig} from 'vite';

import type {StorybookConfig} from '@storybook/react-vite';

const __dirname = dirname(fileURLToPath(import.meta.url));

/**
 * This function is used to resolve the absolute path of a package.
 * It is necessary in projects that use Yarn PnP or are set up within a monorepo.
 */
function getAbsolutePath(value: string) {
  return dirname(fileURLToPath(import.meta.resolve(`${value}/package.json`)));
}
// (dev)/ stories are excluded by default so dev-only code stays out of the Storybook bundle (the same
// intent as `__DEV_TOOLS__` dead-code elimination). `VITE__DEV_TOOLS__=true` includes them to inspect
// the dev UI as it would ship. (Storybook's `stories` doesn't reliably honour `!` negation, so the
// default lists the non-(dev) directories explicitly rather than negating `(dev)/`.)
const includeDevTools = process.env.VITE__DEV_TOOLS__ === 'true';

const STORY_FILES = '*.story.@(js|jsx|mjs|ts|tsx)';

const config: StorybookConfig = {
  stories: includeDevTools
    ? ['../src/**/*.mdx', `../src/**/${STORY_FILES}`]
    : [
        '../src/**/*.mdx',
        `../src/components/**/${STORY_FILES}`,
        `../src/features/**/${STORY_FILES}`,
        `../src/routes/**/${STORY_FILES}`,
        `../src/shared/**/${STORY_FILES}`,
        `../src/ui/**/${STORY_FILES}`,
      ],
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
    mergeConfig(
      {
        ...config,
        // `kodogu:verify-dev-exclusion` (vite.config.ts) guards the *production app* bundle against
        // leaked (dev)/ modules. The Storybook build can't satisfy it — it aliases
        // `@tauri-apps/api/core` to the (dev)/ Tauri stub (the browser has no Tauri), and in
        // `VITE__DEV_TOOLS__=true` mode it bundles (dev)/ stories on purpose — so drop the guard here.
        // (The default build still keeps (dev)/ *debug* stories out via the `stories` list above.)
        plugins: config.plugins?.filter(
          plugin =>
            !(
              typeof plugin === 'object' &&
              plugin !== null &&
              'name' in plugin &&
              plugin.name === 'kodogu:verify-dev-exclusion'
            ),
        ),
      },
      {
        // The Storybook CLI (`storybook dev`/`build`) builds the preview through this viteFinal, so
        // `__DEV_TOOLS__` is resolved here. (`vitest run` uses the `storybook` project in
        // vite.config.ts, which pins its own value.) Dev-only UI is off by default.
        define: {
          __DEV_TOOLS__: includeDevTools,
        },
        // Tauri IPC isn't available in the browser — stub @tauri-apps/api/core for the CLI build.
        // The stub lives under app/src/(dev)/ so it's reused by the vitest projects too.
        resolve: {
          alias: {
            '@tauri-apps/api/core': resolve(__dirname, '../src/(dev)/tauri/core.stub.ts'),
          },
        },
      },
    ),
};
export default config;
