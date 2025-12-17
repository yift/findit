# Min list method

The `min` (also `minimum`) method is used to find the minimal value within a list.

For example:

```bash
findit  -w 'files.map($file $file.size).min() > 1024 * 1024'
```

will show only the directories that the smallest file is larger than 1024 * 1024 bytes.
