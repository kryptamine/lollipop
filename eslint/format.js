import stylistic from '@stylistic/eslint-plugin';
import prettier from 'eslint-config-prettier/flat';

// What Prettier leaves to ESLint. Last, so nothing after it turns a rule of
// Prettier's back on.
export default [
  prettier,

  {
    // After Prettier's, which turns every stylistic rule off. Prettier keeps a
    // type literal broken once it is, so this one it agrees with.
    files: ['**/*.{ts,tsx}'],
    plugins: { '@stylistic': stylistic },
    rules: {
      // A type is written one property to a line, however few it has; an object
      // is from its second property on.
      '@stylistic/object-curly-newline': [
        'error',
        { TSTypeLiteral: 'always', ObjectExpression: { minProperties: 2, consistent: true } },
      ],
      '@stylistic/object-property-newline': 'error',
      // A statement broken over lines is set apart from the one after it.
      '@stylistic/padding-line-between-statements': [
        'error',
        {
          blankLine: 'always',
          prev: [
            'multiline-const',
            'multiline-let',
            'multiline-expression',
            'multiline-block-like',
          ],
          next: '*',
        },
      ],
    },
  },
];
