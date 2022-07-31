module.exports = {
  env: {
    browser: true,
    es2021: true,
  },
  extends: [
    'airbnb-base',
  ],
  parserOptions: {
    ecmaVersion: 'latest',
    sourceType: 'module',
  },
  globals: {
    intrinsics: true,
  },
  rules: {
    'no-use-before-define': ['error', { functions: false, classes: false }],
  },
};
