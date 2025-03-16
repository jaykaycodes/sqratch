/**
 * NOTE - we use biome for linting and formatting
 * This config is only used to run import sorting and
 * TanStack Router's recommended rules on apps/web.
 */

import pluginRouter from '@tanstack/eslint-plugin-router'
import simpleImportSort from 'eslint-plugin-simple-import-sort'
import tseslint from 'typescript-eslint'

/** @type {import('eslint').Linter.Config[]} */
export default [
  // Import sorting for all files in the monorepo
  {
    files: ['**/*.{js,jsx,mjs,cjs,ts,tsx}'],
    ignores: ['**/node_modules/**', '**/dist/**'],
    languageOptions: {
      parser: tseslint.parser,
      parserOptions: {
        ecmaVersion: 'latest',
        sourceType: 'module',
        ecmaFeatures: {
          jsx: true,
        },
      },
    },
    plugins: {
      'simple-import-sort': simpleImportSort,
    },
    rules: {
      'simple-import-sort/imports': [
        'warn',
        {
          groups: [
            // Side effect imports.
            ['^\\u0000'],
            // Node.js builtins prefixed with `node:`.
            ['^node:'],
            // React packages
            ['^react(\u0000?)$', '^react-dom$'],
            // Other packages.
            // Things that start with a letter (or digit or underscore), or `@` followed by a letter.
            ['^@?\\w'],
            // Internal @hub packages
            ['^@hub/'],
            // Absolute imports and other imports such as Vue-style `@/foo`.
            // Anything not matched in another group.
            ['^'],
            // Relative imports.
            // Anything that starts with a dot.
            ['^\\.'],
          ],
        },
      ],
      'simple-import-sort/exports': 'warn',
    },
  },
  // TanStack Router rules only for the apps/web package
  {
    ...pluginRouter.configs['flat/recommended'][0],
    files: ['apps/web/**/*.{js,jsx,ts,tsx}'],
  },
]
