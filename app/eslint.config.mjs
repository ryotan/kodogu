import {resolve} from 'node:path';

import vitest from '@vitest/eslint-plugin';

import query from '@tanstack/eslint-plugin-query';
import router from '@tanstack/eslint-plugin-router';

import {fixupPluginRules, includeIgnoreFile} from '@eslint/compat';
import eslint from '@eslint/js';
import {defineConfig, globalIgnores} from 'eslint/config';
import react from 'eslint-plugin-react';
import * as hooks from 'eslint-plugin-react-hooks';
import storybook from 'eslint-plugin-storybook';
import * as tslint from 'typescript-eslint';

const gitignore = resolve(import.meta.dirname, '.gitignore');

const name = name => config => ({name, ...config});
const ts = config => ({...config, files: tslint.configs.eslintRecommended.files});

const eslintConfig = defineConfig([
  globalIgnores(['src-tauri', 'src/stories', '!.storybook']),
  includeIgnoreFile(gitignore),
  name('#setup/typescript-react')(
    ts({
      settings: {
        react: {
          version: 'detect',
        },
      },
      languageOptions: {
        parserOptions: {
          projectService: true,
          tsconfigRootDir: import.meta.dirname,
        },
      },
    }),
  ),
  name('eslint/recommended')(eslint.configs.recommended),
  name('react/recommended')(ts({...react.configs.flat.recommended, plugins: {react: fixupPluginRules(react)}})),
  name('react/jsx-runtime')(ts({...react.configs.flat['jsx-runtime'], plugins: {react: fixupPluginRules(react)}})),
  name('vitest/recommended')({
    files: ['**/*.test.ts', '**/*.test.tsx'], // or any other pattern
    plugins: {vitest},
    rules: {...vitest.configs.recommended.rules},
    settings: {vitest: {typecheck: true}},
    languageOptions: {globals: {...vitest.environments.env.globals}},
  }),
  name('#customise/react/recommended')(
    ts({
      rules: {
        'react/destructuring-assignment': 'error',
        'react/function-component-definition': [
          'error',
          {
            namedComponents: 'function-declaration',
            unnamedComponents: 'arrow-function',
          },
        ],
        'react/hook-use-state': 'error',
        'react/jsx-boolean-value': 'error',
        'react/jsx-pascal-case': 'error',
        'react/jsx-sort-props': 'error',
      },
    }),
  ),
  ts(hooks.configs.flat['recommended-latest']),
  ts(tslint.configs.base),
  ts(tslint.configs.eslintRecommended),
  ts(tslint.configs.recommendedTypeChecked.findLast(e => e.name === 'typescript-eslint/recommended-type-checked')),
  ts(tslint.configs.strictTypeChecked.findLast(e => e.name === 'typescript-eslint/strict-type-checked')),
  ts(tslint.configs.stylisticTypeChecked.findLast(e => e.name === 'typescript-eslint/stylistic-type-checked')),
  ...query.configs['flat/recommended'].map(ts),
  ...router.configs['flat/recommended'].map(ts).map(name('tanstack/router/flat/recommended')),
  ...storybook.configs['flat/recommended'],
  ...storybook.configs['flat/csf-strict'],
  name('#disable/eslint/recommended')(
    ts({
      rules: {
        // TypeScript (tsc)
        'no-delete-var': 'off',
        'no-invalid-regexp': 'off',

        // Biome.js
        'constructor-super': 'off',
        'for-direction': 'off',
        'getter-return': 'off',
        'no-async-promise-executor': 'off',
        'no-case-declarations': 'off',
        'no-class-assign': 'off',
        'no-compare-neg-zero': 'off',
        'no-cond-assign': 'off',
        'no-const-assign': 'off',
        'no-constant-condition': 'off',
        'no-control-regex': 'off',
        'no-debugger': 'off',
        'no-dupe-args': 'off',
        'no-dupe-class-members': 'off',
        'no-dupe-keys': 'off',
        'no-duplicate-case': 'off',
        'no-empty': 'off',
        'no-empty-character-class': 'off',
        'no-empty-pattern': 'off',
        'no-ex-assign': 'off',
        'no-extra-boolean-cast': 'off',
        'no-extra-semi': 'off',
        'no-fallthrough': 'off',
        'no-func-assign': 'off',
        'no-global-assign': 'off',
        'no-import-assign': 'off',
        'no-inner-declarations': 'off',
        'no-loss-of-precision': 'off',
        'no-misleading-character-class': 'off',
        'no-mixed-spaces-and-tabs': 'off',
        'no-new-symbol': 'off',
        'no-nonoctal-decimal-escape': 'off',
        'no-obj-calls': 'off',
        'no-octal': 'off',
        'no-prototype-builtins': 'off',
        'no-redeclare': 'off',
        'no-regex-spaces': 'off',
        'no-self-assign': 'off',
        'no-setter-return': 'off',
        'no-shadow-restricted-names': 'off',
        'no-sparse-arrays': 'off',
        'no-this-before-super': 'off',
        'no-undef': 'off',
        'no-unexpected-multiline': 'off',
        'no-unreachable': 'off',
        'no-unsafe-finally': 'off',
        'no-unsafe-negation': 'off',
        'no-unsafe-optional-chaining': 'off',
        'no-unused-labels': 'off',
        'no-unused-vars': 'off',
        'no-useless-catch': 'off',
        'no-with': 'off',
        'require-yield': 'off',
        'use-isnan': 'off',
        'valid-typeof': 'off',
        'prefer-const': 'off',
      },
    }),
  ),
  name('#disable/typescript-eslint/strict-type-checked')(
    ts({
      rules: {
        '@typescript-eslint/restrict-template-expressions': [
          'error',
          {allowNumber: true, allowBoolean: true, allowRegExp: true},
        ],

        // TypeScript (tsc)
        '@typescript-eslint/no-unsafe-argument': 'off',
        '@typescript-eslint/no-use-before-define': 'off',

        // Biome.js
        '@typescript-eslint/no-explicit-any': 'off',
        '@typescript-eslint/no-extra-non-null-assertion': 'off',
        '@typescript-eslint/no-extraneous-class': 'off',
        '@typescript-eslint/no-invalid-void-type': 'off',
        '@typescript-eslint/no-misused-new': 'off',
        '@typescript-eslint/no-namespace': 'off',
        '@typescript-eslint/no-non-null-assertion': 'off',
        '@typescript-eslint/no-this-alias': 'off',
        '@typescript-eslint/no-unnecessary-type-constraint': 'off',
        '@typescript-eslint/no-unsafe-declaration-merging': 'off',
        '@typescript-eslint/no-useless-constructor': 'off',
        '@typescript-eslint/no-useless-empty-export': 'off',
        '@typescript-eslint/no-useless-template-literals': 'off',
        '@typescript-eslint/prefer-as-const': 'off',
        '@typescript-eslint/prefer-literal-enum-member': 'off',
      },
    }),
  ),
  name('#disable/typescript-eslint/stylistic-type-checked')(
    ts({
      rules: {
        // TypeScript (tsc)
        '@typescript-eslint/no-unused-vars': 'off',
        // Biome.js
        '@typescript-eslint/array-type': 'off',
        '@typescript-eslint/consistent-type-definitions': 'off',
        'dot-notation': 'off',
        '@typescript-eslint/dot-notation': 'off',
        '@typescript-eslint/no-confusing-non-null-assertion': 'off',
        '@typescript-eslint/no-empty-function': 'off',
        '@typescript-eslint/no-empty-interface': 'off',
        '@typescript-eslint/no-inferrable-types': 'off',
        '@typescript-eslint/prefer-for-of': 'off',
        '@typescript-eslint/prefer-function-type': 'off',
        '@typescript-eslint/prefer-namespace-keyword': 'off',
        '@typescript-eslint/prefer-optional-chain': 'off',
        // そもそも不要
        '@typescript-eslint/ban-tslint-comment': 'off', // tslintは使っていない
        '@typescript-eslint/consistent-type-assertions': 'off', // そもそも型アサーションはasしか使わない
      },
    }),
  ),
  name('#disable/typescript-eslint/too-inconvenient')(
    ts({rules: {'@typescript-eslint/no-confusing-void-expression': 'off'}}),
  ),
  name('#disable/react/recommended')(
    ts({rules: {'react/prop-types': 'off', 'react/jsx-no-undef': 'off', 'react/jsx-key': 'off'}}),
  ),
  // Production code throws should pick a typed error (ApplicationError or PlatformError) so the
  // failure category is visible at the throw site. Test/story/fixture files are exempt because
  // `new Error('test value')` is a common, low-noise pattern there.
  name('#customise/no-raw-error')(
    ts({
      rules: {
        'no-restricted-syntax': [
          'error',
          {
            selector: 'NewExpression[callee.name="Error"]',
            message: 'Use ApplicationError or PlatformError from #shared/core/errors instead of raw Error.',
          },
        ],
      },
    }),
  ),
  name('#disable/no-raw-error/test-and-fixture')({
    files: ['**/*.test.{ts,tsx}', '**/*.spec.{ts,tsx}', '**/*.story.{ts,tsx}', '**/*.fixture.{ts,tsx}'],
    rules: {'no-restricted-syntax': 'off'},
  }),
  // Query factories must go through the call-site-aware wrapper (#shared/core/query/queryOptions),
  // which attaches a QueryCallSiteError cause. Importing TanStack's queryOptions directly bypasses it.
  name('#customise/no-direct-tanstack-query-options')(
    ts({
      rules: {
        'no-restricted-imports': [
          'error',
          {
            paths: [
              {
                name: '@tanstack/react-query',
                importNames: ['queryOptions'],
                message: 'Import queryOptions from #shared/core/query/queryOptions instead of @tanstack/react-query.',
              },
            ],
          },
        ],
      },
    }),
  ),
  // The wrapper itself must import the real implementation.
  name('#disable/no-direct-tanstack-query-options/wrapper')({
    files: ['**/shared/core/query/queryOptions.ts'],
    rules: {'no-restricted-imports': 'off'},
  }),
]);

export default eslintConfig;
