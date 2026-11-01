import js from '@eslint/js';
import tseslint from 'typescript-eslint';
import regexp from 'eslint-plugin-regexp';
import promise from 'eslint-plugin-promise';
import sonarjs from 'eslint-plugin-sonarjs';
import unicorn from 'eslint-plugin-unicorn';
import importX from 'eslint-plugin-import-x';
import checkFile from 'eslint-plugin-check-file';
import perfectionist from 'eslint-plugin-perfectionist';
import comments from '@eslint-community/eslint-plugin-eslint-comments';

import { namedUnionMembers, repoRoot, upADirectory } from './shared.js';

// What every file is held to.
export default [
  {
    files: ['**/*.{ts,tsx,js,mjs,cjs}'],
    extends: [
      js.configs.recommended,
      tseslint.configs.strictTypeChecked,
      tseslint.configs.stylisticTypeChecked,
      importX.flatConfigs.recommended,
      importX.flatConfigs.typescript,
      unicorn.configs.recommended,
      sonarjs.configs.recommended,
      promise.configs['flat/recommended'],
      regexp.configs['flat/recommended'],
    ],
    plugins: { perfectionist, 'eslint-comments': comments },
    linterOptions: { reportUnusedDisableDirectives: 'error' },
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: repoRoot,
      },
    },
    settings: {
      'import-x/resolver': {
        typescript: { project: `${repoRoot}/tsconfig.json` },
      },
    },
    rules: {
      // An exception must state its reason (ADR 0001).
      'eslint-comments/require-description': 'error',
      complexity: ['error', 10],
      'sonarjs/cognitive-complexity': ['error', 10],
      'max-depth': ['error', 3],
      'max-lines': ['error', { max: 200, skipBlankLines: true, skipComments: true }],
      'max-lines-per-function': ['error', 25],
      'max-params': ['error', 4],
      // @typescript-eslint/no-deprecated reports the same, at a fifth of the cost.
      'sonarjs/deprecation': 'off',
      // A type says what a value is. An interface also declares itself open to
      // merging, which is a promise nothing here wants to make.
      '@typescript-eslint/consistent-type-definitions': ['error', 'type'],
      // A case added to a union fails every switch over it, which a default
      // would swallow.
      '@typescript-eslint/switch-exhaustiveness-check': [
        'error',
        { allowDefaultCaseForExhaustiveSwitch: false, considerDefaultExhaustiveForUnions: false },
      ],
      '@typescript-eslint/consistent-type-imports': ['error', { fixStyle: 'inline-type-imports' }],
      '@typescript-eslint/explicit-function-return-type': [
        'error',
        { allowExpressions: true, allowTypedFunctionExpressions: true },
      ],
      '@typescript-eslint/no-unused-vars': [
        'error',
        { argsIgnorePattern: '^_', varsIgnorePattern: '^_' },
      ],
      // Node's own, then packages, then what sits beside the file. Inside a group
      // the shorter import comes first.
      'import-x/order': 'off',
      'perfectionist/sort-imports': [
        'error',
        {
          type: 'line-length',
          order: 'asc',
          fallbackSort: { type: 'alphabetical', order: 'asc' },
          groups: [
            'builtin',
            'external',
            'internal',
            ['parent', 'sibling', 'index'],
            // Never moved, because a side effect's place can matter.
            ['side-effect-style', 'side-effect'],
            'unknown',
          ],
          newlinesBetween: 1,
        },
      ],
      'import-x/no-default-export': 'error',
      // TypeScript answers these three, and they walk every imported module again.
      'import-x/default': 'off',
      'import-x/namespace': 'off',
      'import-x/no-named-as-default-member': 'off',
      '@typescript-eslint/no-restricted-imports': ['error', { patterns: [upADirectory] }],
      // Return early: what follows a return needs no else around it.
      'no-else-return': ['error', { allowElseIf: false }],
      // Arrow functions everywhere. allowUnboundThis leaves the one case that
      // needs the keyword: a callback that is called with a `this`.
      'func-style': ['error', 'expression', { allowArrowFunctions: true }],
      'prefer-arrow-callback': ['error', { allowNamedFunctions: false, allowUnboundThis: true }],
      // Assignments and renames stay as written: they read worse destructured.
      'prefer-destructuring': 'off',
      '@typescript-eslint/prefer-destructuring': [
        'error',
        {
          VariableDeclarator: { object: true, array: true },
          AssignmentExpression: { object: false, array: false },
        },
      ],
      // Names here come from GLOSSARY.md and from the frameworks, which the
      // replacement lists know neither of.
      'unicorn/name-replacements': 'off',
      'unicorn/prevent-abbreviations': 'off',
      'unicorn/filename-case': ['error', { cases: { kebabCase: true, pascalCase: true } }],
      // Returning early keeps the short branch first; this folds every such pair.
      'unicorn/prefer-ternary': 'off',
      'no-restricted-syntax': ['error', namedUnionMembers],
    },
  },

  {
    // The repository writes .test.ts, never .spec.ts.
    files: ['**/*.{ts,tsx}'],
    plugins: { 'check-file': checkFile },
    rules: {
      'check-file/filename-blocklist': [
        'error',
        { '**/*.spec.ts': '*.test.ts', '**/*.spec.tsx': '*.test.tsx' },
      ],
    },
  },

  {
    // A constant a module declares at its top is written as one: SERVER_LIMIT,
    // not serverLimit. Functions and objects keep their own case.
    files: ['**/*.{ts,tsx}'],
    rules: {
      '@typescript-eslint/naming-convention': [
        'error',
        {
          selector: 'variable',
          modifiers: ['const', 'global'],
          types: ['boolean', 'string', 'number'],
          format: ['UPPER_CASE'],
        },
      ],
    },
  },
];
