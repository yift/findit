# Is true (`IS [NOT] TRUE`) operator

The is true (`IS TRUE`) and is not true (`IS NOT TRUE`) are used to verify that a Boolean operand is true (or not).
Note that empty `IS TRUE` will return false and empty `IS NOT TRUE` will return true.

For example:

```bash
findit -w 'content.contains("test") IS NOT TRUE'
```

Will show all the files that have not `test` in their content and all the files which their content is unreadable (for example, directories).
