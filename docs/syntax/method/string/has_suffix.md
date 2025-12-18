# HasSuffix string method

The `hasSuffix` (or `endsWith`, or `has_suffix` or `ends_with`)  method is used to check if a string ends with another string.
It accept a single argument which is the substring to check for.

For example:

```bash
findit -w 'stem.hasSuffix("527")'
```

will show the files with name (without extension) that ends with the number "527".
