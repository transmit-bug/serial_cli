export default {
  extends: ['@commitlint/config-conventional'],
  rules: {
    'type-enum': [2, 'always', [
      'feat', 'fix', 'docs', 'style', 'refactor',
      'perf', 'test', 'chore', 'ci', 'Update'
    ]],
    // warn (not error) — scopes outside this list are common (release, deps bumps)
    'scope-enum': [1, 'always', [
      'cli', 'gui', 'serial', 'protocol', 'lua',
      'task', 'config', 'build', 'ci', 'deps', 'backend'
    ]],
    'type-case': [2, 'always', 'lower-case'],
    'subject-case': [0],
    'subject-empty': [2, 'never'],
    'subject-full-stop': [2, 'never', '.'],
    'header-max-length': [2, 'always', 100],
    'body-max-line-length': [2, 'always', 150]
  }
};
