import eslint from '@cloudpss/eslint-config';

export default eslint(
  {
    ignores: ['packages/monaco/src/monaco-api.js'],
  },
  {
    files: ['packages/website/**/*'],
    rules: {
      'unicorn/filename-case': 'off',
    },
  },
);
