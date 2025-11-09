import vitest from '@vitest/eslint-plugin';

import query from '@tanstack/eslint-plugin-query';
import router from '@tanstack/eslint-plugin-router';

import {includeIgnoreFile} from '@eslint/compat';
import eslint from '@eslint/js';
import {defineConfig, globalIgnores} from 'eslint/config';
import esimport from 'eslint-plugin-import';
import react from 'eslint-plugin-react';
import * as hooks from 'eslint-plugin-react-hooks';
import storybook from 'eslint-plugin-storybook';
import {resolve} from 'node:path';
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
  name('react/recommended')(ts(react.configs.flat.recommended)),
  name('react/jsx-runtime')(ts(react.configs.flat['jsx-runtime'])),
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
            namedComponents: 'arrow-function',
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
  ts(hooks.configs['recommended-latest']),
  ts(tslint.configs.base),
  ts(tslint.configs.eslintRecommended),
  ts(tslint.configs.recommendedTypeChecked.findLast(e => e.name === 'typescript-eslint/recommended-type-checked')),
  ts(tslint.configs.strictTypeChecked.findLast(e => e.name === 'typescript-eslint/strict-type-checked')),
  ts(tslint.configs.stylisticTypeChecked.findLast(e => e.name === 'typescript-eslint/stylistic-type-checked')),
  ...query.configs['flat/recommended'].map(ts),
  ...router.configs['flat/recommended'].map(ts).map(name('tanstack/router/flat/recommended')),
  name('#setup/import')({
    plugins: {import: esimport},
    settings: {
      'import/internal-regex': '^!?#',
      'import/extensions': ['.ts', '.cts', '.mts', '.tsx', '.js', '.jsx', '.mjs', '.cjs'],
      'import/external-module-folders': ['node_modules', 'node_modules/@types'],
      'import/parsers': {'@typescript-eslint/parser': ['.ts', '.cts', '.mts', '.tsx']},
      'import/resolver': {node: {extensions: ['.ts', '.cts', '.mts', '.tsx', '.js', '.jsx', '.mjs', '.cjs']}},
    },
    rules: {'import/named': 'off'},
  }),
  ...storybook.configs['flat/recommended'],
  ...storybook.configs['flat/csf-strict'],
  name('#customise/typescript-eslint/stylistic-type-checked')(
    ts({
      rules: {
        // https://typescript-eslint.io/rules/consistent-type-definitions
        '@typescript-eslint/consistent-type-definitions': ['error', 'type'],
      },
    }),
  ),
  name('#customise/import/extraneous-dependencies')({
    rules: {
      // https://github.com/import-js/eslint-plugin-import/blob/main/docs/rules/no-extraneous-dependencies.md
      'import/no-extraneous-dependencies': [
        'error',
        {
          devDependencies: [
            '**/*.test.*',
            '**/*.spec.*',
            '**/_dev/**',
            '**/(dev)/**',
            '**/eslint.config.mjs',
            '**/vite.config.ts',
            '**/vitest.workspace.ts',
            '.storybook/vitest.setup.ts',
          ],
        },
      ],
    },
  }),
  name('#customise/import/order')({
    rules: {
      // https://github.com/import-js/eslint-plugin-import/blob/main/docs/rules/order.md
      'import/order': [
        'error',
        {
          groups: [['builtin', 'external'], 'internal', 'parent', ['index', 'sibling']],
          'newlines-between': 'always',
          alphabetize: {
            order: 'asc',
          },
          named: {
            enabled: true,
            types: 'types-first',
          },
          pathGroups: [
            {
              pattern: '{vite*,vite*/**,@vite*/**}',
              group: 'external',
              position: 'before',
            },
            {
              pattern: '{react,react-dom,react-dom*/**}',
              group: 'external',
              position: 'before',
            },
            {
              pattern: '{tanstack*,tanstack*/**,@tanstack*/**}',
              group: 'external',
              position: 'before',
            },
            {
              pattern: '{radix-ui*,radix-ui*/**,@radix-ui*/**}',
              group: 'external',
              position: 'before',
            },
          ],
          pathGroupsExcludedImportTypes: ['builtin'],
        },
      ],
    },
  }),
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
]);

export default eslintConfig;
