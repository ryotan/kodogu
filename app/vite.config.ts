import path from 'node:path';
import {fileURLToPath} from 'node:url';

import react from '@vitejs/plugin-react';
import {playwright} from '@vitest/browser-playwright';
import type {Plugin} from 'vite';
import {defineConfig} from 'vite';
import paths from 'vite-tsconfig-paths';

import router from '@tanstack/router-plugin/vite';

import {storybookTest} from '@storybook/addon-vitest/vitest-plugin';

// Fails the production app build if any module under `src/(dev)/` ends up in the emitted bundle.
// The dev-only branches in app code are gated by `__DEV_TOOLS__` ternaries so terser should
// dead-code-eliminate them, but this guards against (1) a future change forgetting the gate
// and (2) a non-static guard that terser cannot prove unreachable.
//
// This guards the *production app* bundle only. The Storybook build drops it (see .storybook/main.ts):
// it aliases `@tauri-apps/api/core` to the (dev)/ Tauri stub and, in dev-tools mode, bundles (dev)/
// stories on purpose, so (dev)/ modules are expected there.
const verifyDevExclusion = (): Plugin => ({
  name: 'kodogu:verify-dev-exclusion',
  // `post` so the check runs after every other plugin has had a chance to touch the bundle —
  // we want to inspect what would actually ship.
  enforce: 'post',
  apply: 'build',
  generateBundle(_, bundle) {
    const leaked = Object.values(bundle).flatMap(chunk =>
      chunk.type === 'chunk'
        ? Object.keys(chunk.modules)
            .filter(moduleId => moduleId.includes('(dev)'))
            .map(moduleId => `${chunk.fileName} <- ${moduleId}`)
        : [],
    );
    if (leaked.length > 0) {
      this.error(`\`(dev)\` modules leaked into production bundle:\n${leaked.map(l => `  - ${l}`).join('\n')}`);
    }
  },
});

const VITE_DEFAULT_PORT = 5173;

const dirname = path.dirname(fileURLToPath(import.meta.url));

// Stub for `@tauri-apps/api/core`: browser-based vitest projects don't have Tauri runtime, so any
// `invoke()` call on the real module fails with `Cannot read properties of undefined (reading 'invoke')`.
// Storybook applies the same stub via `.storybook/main.ts`; this constant re-uses it for the
// `browser` and `browser-devtools` projects below. Per-test `vi.mock('@tauri-apps/api/core', ...)`
// still takes priority over this alias when a test wants to assert specific Tauri behaviour.
const tauriCoreStub = path.resolve(dirname, 'src/(dev)/tauri/core.stub.ts');

const {TAURI_DEV_HOST: host, TAURI_DEV_SERVER_PORT: portString} = process.env;

// Configurations for mobile app development.
// Tauri CLI adds environment variables to the dev-server process when running Android app.
const port =
  portString && Number.isSafeInteger(Number.parseInt(portString, 10))
    ? Number.parseInt(portString, 10)
    : VITE_DEFAULT_PORT;

const defaultBrowser = () => ({
  enabled: true,
  headless: true,
  screenshotFailures: true,
  provider: playwright(),
  instances: [{browser: 'chromium' as const}],
});

// https://vitejs.dev/config/
export default defineConfig(({mode}) => ({
  plugins: [
    paths(),
    router({
      target: 'react',
      autoCodeSplitting: true,
      generatedRouteTree: './gen/router/routes.ts',
      routeFileIgnorePattern: '\\.(spec|test|story)\\.[tj]sx?$',
    }),
    react(),
    verifyDevExclusion(),
  ],

  // Build-time flag for dev-only UI (devtools panels, stack-trace UI, etc.).
  // Defaults to true in dev server, false in production builds. Vitest projects and Storybook
  // override this per-environment so that tests and stories run with dev tools disabled by default.
  define: {
    __DEV_TOOLS__: JSON.stringify(mode !== 'production'),
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: port,
    strictPort: true,
    host: host ?? false,
    // hmr host and port of ws client in mobile apps is not set by default, so we need to set it manually
    hmr: host ? {host, port} : undefined,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ['**/src-tauri/**'],
    },
  },
  test: {
    coverage: {
      include: ['src/**/*.{ts,tsx}'],
      exclude: [
        '**/*.d.ts',
        '**/*.test.{ts,tsx}',
        '**/*.spec.{ts,tsx}',
        '**/*.story.{ts,tsx}',
        '**/*.fixture.{ts,tsx}',
        '**/*.mock.{ts,tsx}',
        '**/[(]dev[)]/**',
        '**/[(]prototype[)]/**',
        '**/main.tsx',
        '**/stories/**',
        '**/.storybook/**',
      ],
    },
    projects: [
      {
        test: {
          name: 'unit',
          include: ['src/**/*.test.{ts,tsx}'],
        },
        extends: 'vite.config.ts',
        define: {
          __DEV_TOOLS__: 'false',
        },
      },
      {
        test: {
          name: 'browser',
          include: ['src/**/*.spec.tsx'],
          exclude: ['src/**/*.devtools.spec.tsx'],
          browser: defaultBrowser(),
        },
        extends: 'vite.config.ts',
        define: {
          __DEV_TOOLS__: 'false',
        },
        resolve: {alias: {'@tauri-apps/api/core': tauriCoreStub}},
      },
      {
        test: {
          name: 'browser-devtools',
          include: ['src/**/*.devtools.spec.tsx'],
          browser: defaultBrowser(),
        },
        extends: 'vite.config.ts',
        define: {
          __DEV_TOOLS__: 'true',
        },
        resolve: {alias: {'@tauri-apps/api/core': tauriCoreStub}},
      },
      {
        test: {
          name: 'storybook',
          browser: defaultBrowser(),
          setupFiles: ['.storybook/preview/vitest.setup.ts'],
        },
        extends: 'vite.config.ts',
        // Stories run through two entry points: the Storybook CLI (.storybook/main.ts's viteFinal) and
        // — here — `vitest run` via addon-vitest's storybookTest. This project pins dev tools off for
        // the `vitest run` path.
        define: {
          __DEV_TOOLS__: 'false',
        },
        // Tauri IPC isn't available in the browser; stub it for the `vitest run` storybook project.
        resolve: {alias: {'@tauri-apps/api/core': tauriCoreStub}},
        plugins: [
          // The plugin will run tests for the stories defined in your Storybook config
          // See options at: https://storybook.js.org/docs/writing-tests/test-addon#storybooktest
          storybookTest({configDir: path.join(dirname, '.storybook')}),
        ],
      },
    ],
  },
}));
