# Env function
The `env` (or `environment`) function will accept one string argument and will return the environment variable (or empty if not such variable exists).


For example:
```bash
findit -w 'env("USER") != owner'
```
Will list the files that are not owned by the current user.
