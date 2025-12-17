# As String (`AS STRING`) operator

The `AS STRING`  (or `AS TEXT`, or `AS STR`) operator is used to cast an operand to a string.

For example:

```bash
findit -d 'file: `path` modified at `(modified AS STRING).skip(7).take(4)`'
```

Will show all the files and the year in which they have been modified
