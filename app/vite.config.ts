import path from 'node:path';
import {fileURLToPath} from 'node:url';

import react from '@vitejs/plugin-react';
import {playwright} from '@vitest/browser-playwright';
import {defineConfig} from 'vite';
import paths from 'vite-tsconfig-paths';
import type {TestCase, TestModule, TestSuite} from 'vitest/node';

import router from '@tanstack/router-plugin/vite';

import {storybookTest} from '@storybook/addon-vitest/vitest-plugin';

const VITE_DEFAULT_PORT = 5173;

const dirname = path.dirname(fileURLToPath(import.meta.url));

const {TAURI_DEV_HOST: host, TAURI_DEV_SERVER_PORT: portString} = process.env;

// Configurations for mobile app development.
// Tauri CLI adds environment variables to the dev-server process when running Android app.
const port =
  portString && Number.isSafeInteger(Number.parseInt(portString, 10))
    ? Number.parseInt(portString, 10)
    : VITE_DEFAULT_PORT;

// https://vitejs.dev/config/
export default defineConfig(() => ({
  plugins: [
    paths(),
    router({
      target: 'react',
      autoCodeSplitting: true,
      generatedRouteTree: './gen/router/routes.ts',
      routeFileIgnorePattern: '\\.(spec|test|story)\\.[tj]sx?$',
    }),
    react(),
  ],

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
    onConsoleLog(log: string, type: 'stdout' | 'stderr', entity: TestCase | TestModule | TestSuite | undefined) {
      if (entity?.type !== 'test' || type !== 'stderr') {
        return;
      }
      if (entity.module.moduleId.endsWith('/reportError.spec.ts') && log.startsWith('reportError ')) {
        return false;
      }
      if (entity.module.moduleId.endsWith('/PopoverRouteErrorPage.spec.tsx')) {
        const expectedFailures: Readonly<Record<string, string | undefined>> = {
          'reruns a failed loader on Try Again': 'loader failed',
          'refetches a suspense query that failed while rendering on Try Again': 'query failed',
          'logs the error when reloading the routes fails on Try Again': 'loader failed',
        };
        const expectedFailure = expectedFailures[entity.name];
        if (
          expectedFailure &&
          ((log.includes(`[Error: ${expectedFailure}]`) && log.includes('React will try to recreate')) ||
            log.trim() === 'Warning: Error in route match: __root__/')
        ) {
          return false;
        }
      }
    },
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
        '**/.storybook/**',
      ],
    },
    projects: [
      {
        extends: 'vite.config.ts',
        test: {
          name: 'unit',
          include: ['src/**/*.test.{ts,tsx}'],
        },
      },
      {
        extends: 'vite.config.ts',
        // Vitest reports test console output; disable Vite's duplicate development-server forwarding.
        server: {forwardConsole: false},
        test: {
          name: 'browser',
          include: ['src/**/*.spec.{ts,tsx}'],
          browser: {
            enabled: true,
            headless: true,
            screenshotFailures: false,
            provider: playwright(),
            instances: [{browser: 'chromium' as const}],
          },
        },
      },
      {
        extends: 'vite.config.ts',
        plugins: [
          // The plugin will run tests for the stories defined in your Storybook config
          // See options at: https://storybook.js.org/docs/writing-tests/test-addon#storybooktest
          storybookTest({configDir: path.join(dirname, '.storybook')}),
        ],
        test: {
          name: 'storybook',
          browser: {
            enabled: true,
            headless: true,
            provider: playwright(),
            instances: [{browser: 'chromium' as const}],
          },
        },
      },
    ],
  },
}));
