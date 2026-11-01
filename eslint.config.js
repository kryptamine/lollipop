import { defineConfig, globalIgnores } from 'eslint/config';

import base from './eslint/base.js';
import view from './eslint/view.js';
import format from './eslint/format.js';
import scripts from './eslint/scripts.js';

// The rules are a file to a concern under eslint/. The order is kept: a later
// block replaces a rule an earlier one set for the same file.
export default defineConfig(
  globalIgnores([
    'dist/**',
    'target/**',
    'crates/**',
    // Research and development tools run by hand, which ship nothing.
    'scripts/**',
  ]),

  base,
  view,
  scripts,
  format,
);
