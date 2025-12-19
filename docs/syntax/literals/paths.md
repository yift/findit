# Paths Literals

To use a path literal in an expression, use the `@` sign before the name of the file. For example:

```bash
findit -w 'parent = @src .absolute'
```

will find all the files whose parents have the same absolute path as the src (i.e. equivalent to `ls ./src`).

You can use quotes when the file name contains spaces or to avoid adding a space after the filename if you need to access it. For example:

```bash
findit -w 'content = @"src/main.rs".content'
```

will find all the files that have the same content as `src/main.rs`.
