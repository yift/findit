# Avg list method

The `avg` (also `average`) method is used to find the average value within a list of numbers.

For example:

```bash
findit  -w 'files.map($file $file.size).avg() > 1024 * 1024'
```

will show only the directories that the average file is larger than 1024 * 1024 bytes.
