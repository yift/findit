# RemovePrefix string method

The `removePrefix` (or `remove_prefix`)  method is used create another string without a prefix.
It accept a single argument which is the substring to remove.
If the string is not starting with the prefix, it will return the original string.

For example:

```bash
findit -w 'stem.removePrefix("temp-") = "hello"'
```

will show the files with name that named `hello.*` and `temp-hello.*`.
