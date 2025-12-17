# Distinct list method

The `distinct` (also `unique`) method is used to remove duplicates from a list.

For example:

```bash
findit  -w 'extension = "txt" AND words().distinct().length() = words().length()'
```

will show all the text files that has no duplicate words.
