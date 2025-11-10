/// <reference types="@vitest/browser/providers/playwright" />
import {defineWorkspace} from 'vitest/config';

import {storybookTest} from '@storybook/experimental-addon-test/vitest-plugin';
// @ts-expect-error -- node:path is Node.js module
import path from 'node:path';
// @ts-expect-error -- node:path is Node.js module
import {fileURLToPath} from 'node:url';

// @ts-expect-error -- node:path is Node.js module
// eslint-disable-next-line @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-assignment
const dirname = path.dirname(fileURLToPath(import.meta.url));

// More info at: https://storybook.js.org/docs/writing-tests/test-addon
export default defineWorkspace([
      'vite.config.ts',
      {
        extends: 'vite.config.ts',
        plugins: [
          // The plugin will run tests for the stories defined in your Storybook config
          // See options at: https://storybook.js.org/docs/writing-tests/test-addon#storybooktest
          // eslint-disable-next-line @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-assignment
          storybookTest({configDir: path.join(dirname, '.storybook')}),
        ],
        test: {
          name: 'storybook',
          browser: {
            enabled: true,
            headless: true,
            instances: [{browser: 'chromium'}],
            provider: 'playwright',
          },
          setupFiles: ['.storybook/vitest.setup.ts'],
        },
      },
    ],
);
