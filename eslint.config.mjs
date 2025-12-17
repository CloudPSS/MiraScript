import eslint from '@cloudpss/eslint-config';

export default eslint({
  files: ['packages/website/**/*'],
  rules: {
    'unicorn/filename-case': 'off',
  },
});
