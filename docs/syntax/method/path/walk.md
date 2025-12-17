# Walk path method

The `walk`  method is used to convert a file to all the files (including sub directories).

For example:

```bash
findit -w 'walk().filter($f $f.extension = "txt").length() > 100'
```

will show all the directories that have more than 100 offsprings.
