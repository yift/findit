# Max list method

**Aliases:** `maximum()`

The `max` method is used to find the maximal value within a list.

For example:

```bash
findit  -w 'files.map($file $file.size).max() > 1024 * 1024'
```

will show only the directories that the largest file is larger than 1024 * 1024 bytes.
