# Sort list method

The `sort` (also `order`) method is used to sort the arguments in a list.

For example:

```bash
findit  -d 'name: `name` first 5 files: `files.map($file $file.name).sort().take(5)`' -w 'IS DIR'
```

will show all the directories with the first five files (order by name).
