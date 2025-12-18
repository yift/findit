# RemoveSuffix string method

The `removeSuffix` (or `remove_suffix`)  method is used create another string without a suffix.
It accept a single argument which is the substring to remove.
If the string is not ending with the suffix, it will return the original string.

For example:

```bash
findit -w 'name.removeSuffix(".bak").hasSuffix(".txt")'
```

will show the files with name that named `*.txt` and `*.txt.bak`.
