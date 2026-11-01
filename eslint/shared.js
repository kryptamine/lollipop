import path from 'node:path';

// Every path a plugin reads is anchored here: an editor runs ESLint from
// wherever the file is.
export const repoRoot = path.dirname(import.meta.dirname);

// Walking up out of a directory says nothing about what is being reached for.
export const upADirectory = {
  group: ['..', '../*', '../**'],
  message: 'Import from ./, never up a directory.',
};

// Each scoped no-restricted-syntax carries this too, since a later block
// replaces the rule for its files.
export const namedUnionMembers = {
  selector: 'TSUnionType > TSTypeLiteral',
  message: 'Name each member of a union as a type of its own, then join the names.',
};
