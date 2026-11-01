import globals from 'globals';
import jsxA11y from 'eslint-plugin-jsx-a11y';
import reactHooks from 'eslint-plugin-react-hooks';
import eslintReact from '@eslint-react/eslint-plugin';
import reactRefresh from 'eslint-plugin-react-refresh';

import { namedUnionMembers, upADirectory } from './shared.js';

// The view: Preact, its hooks, and the one module that reaches Tauri.
export default [
  {
    files: ['ui/**/*.{ts,tsx}'],
    extends: [
      jsxA11y.flatConfigs.strict,
      eslintReact.configs['strict-type-checked'],
      // The hook rules belong to eslint-plugin-react-hooks.
      eslintReact.configs['disable-conflict-eslint-plugin-react-hooks'],
    ],
    languageOptions: { globals: globals.browser },
    plugins: { 'react-hooks': reactHooks, 'react-refresh': reactRefresh },
    // Preact's hooks and JSX follow the React 19 rules these plugins check.
    settings: { 'react-x': { importSource: 'preact', version: '19.0.0' } },
    rules: {
      // Spread rather than extended: the plugin ships this config with the
      // pre-flat shape for `plugins`.
      ...reactHooks.configs['recommended-latest'].rules,
      'react-refresh/only-export-components': ['error', { allowConstantExport: true }],
      // A component returns an element and the compiler knows it.
      '@typescript-eslint/explicit-function-return-type': 'off',
      '@typescript-eslint/restrict-template-expressions': [
        'error',
        {
          allowAny: false,
          allowBoolean: false,
          allowNever: false,
          allowNullish: false,
          allowNumber: true,
          allowRegExp: false,
        },
      ],
      '@typescript-eslint/no-restricted-imports': [
        'error',
        {
          patterns: [
            upADirectory,
            { group: ['@tauri-apps/*'], message: 'Reach Tauri through ui/src/bridge.ts.' },
          ],
        },
      ],
      'no-restricted-syntax': [
        'error',
        namedUnionMembers,
        {
          selector:
            'JSXAttribute > JSXExpressionContainer > :function > BlockStatement[body.length>1]',
          message: 'Name a handler of more than one statement: onChange={handleChange}.',
        },
      ],
    },
  },

  {
    files: ['ui/src/bridge.ts'],
    rules: {
      '@typescript-eslint/no-restricted-imports': ['error', { patterns: [upADirectory] }],
    },
  },
];
