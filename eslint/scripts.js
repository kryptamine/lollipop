import tseslint from 'typescript-eslint';

// What is no source of the view: a tool's config and ESLint's own rules.
export default [
  {
    files: ['**/*.config.ts'],
    rules: { 'import-x/no-default-export': 'off' },
  },

  {
    // A config is the call its tool asks for, and that call is the side effect
    // the rule names.
    files: ['**/*.config.{ts,js}', 'eslint/*.js'],
    rules: { 'unicorn/no-top-level-side-effects': 'off' },
  },

  {
    // No .js file is in a tsconfig, so no rule here can be type-aware.
    files: ['**/*.js'],
    extends: [tseslint.configs.disableTypeChecked],
    rules: {
      '@typescript-eslint/explicit-function-return-type': 'off',
      'import-x/no-default-export': 'off',
      'import-x/no-named-as-default': 'off',
      'import-x/no-named-as-default-member': 'off',
    },
  },
];
