import {existsSync, readFileSync} from 'node:fs';
import path from 'node:path';

import react from '@vitejs/plugin-react';
import {playwright} from '@vitest/browser-playwright';
import type {ViteUserConfig} from 'vitest/config';
import {defineConfig} from 'vitest/config';

import {storybookTest} from '@storybook/addon-vitest/vitest-plugin';

// The Vitest settings that every package with tests shares.

type Test = NonNullable<ViteUserConfig['test']>;
type Project = Exclude<NonNullable<Test['projects']>[number], string>;

export const coverageExclude = [
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
];

// Each project gets an object of its own: Vitest derives a project for each browser instance from it.
export const browser = (): NonNullable<Test['browser']> => ({
  enabled: true,
  headless: true,
  screenshotFailures: true,
  provider: playwright(),
  instances: [{browser: 'chromium'}],
});

// The unit and browser test projects of the package in `directory`, and the story tests when it has a Storybook.
// They are named after the package, which is unique in the workspace; the directory's name is not (`tools/*/desktop`).
export function packageProjects(directory: string): Project[] {
  const {name} = JSON.parse(readFileSync(path.join(directory, 'package.json'), 'utf8')) as {name: string};
  const configDir = path.join(directory, '.storybook');
  return [
    {plugins: [react()], root: directory, test: {name: `${name} unit`, include: ['src/**/*.test.{ts,tsx}']}},
    {
      plugins: [react()],
      root: directory,
      test: {name: `${name} browser`, include: ['src/**/*.spec.{ts,tsx}'], browser: browser()},
    },
    ...(existsSync(configDir)
      ? [
          {
            plugins: [storybookTest({configDir})],
            root: directory,
            test: {name: `${name} storybook`, browser: browser()},
          },
        ]
      : []),
  ];
}

// The configuration with which a package runs its own tests and measures its own coverage. A package without tests
// passes `{passWithNoTests: true}`, so that it still appears, uncovered, in the coverage of every package.
export function definePackageConfig(directory: string, {passWithNoTests = false} = {}): ViteUserConfig {
  return defineConfig({
    root: directory,
    test: {
      passWithNoTests,
      coverage: {include: ['src/**/*.{ts,tsx}'], exclude: coverageExclude},
      projects: packageProjects(directory),
    },
  });
}
