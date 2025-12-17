# Sub (`/`) path operator

The Sub (`/`) sub operator will return a file within a directory.
It can be used as a binary operator like: `parent / "build"` that is the build sibling directory of the current file.
It can be used as a postfix unary operator like `/ "build"` that is the build child directory of the current file (equivalent to (`me / "build"`)).

For example:

```bash
findit -w '(parent / "build").IS DIR'
```

Will filter all the files that have a sibling build directory.
