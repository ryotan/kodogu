import {globSync} from 'node:fs';
import path from 'node:path';

import {defineConfig} from 'vitest/config';

import {coverageExclude, packageProjects} from '@kodogu/vitest-config';

// Merges the blob reports that each package writes (`test:coverage:blob`) into one coverage report of every package.
// The tests run in the packages; the projects are declared here only so that the merge knows them.
const packagesWithTests = globSync(['apps/desktop/*/vitest.config.mjs', 'tools/*/desktop/vitest.config.mjs'], {
  cwd: import.meta.dirname,
})
  .map(file => path.dirname(file))
  .sort();

export default defineConfig({
  test: {
    coverage: {
      include: ['apps/desktop/*/src/**/*.{ts,tsx}', 'tools/*/desktop/src/**/*.{ts,tsx}'],
      exclude: coverageExclude,
      reportsDirectory: 'coverage',
    },
    projects: packagesWithTests.flatMap(directory => packageProjects(path.join(import.meta.dirname, directory))),
  },
});
