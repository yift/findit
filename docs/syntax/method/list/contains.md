# Contains list method

The `contains`  method is used to check if a list contains an item.
It accept a single argument which is the item to check for.

For example:

```bash
findit -w 'files.map($f $f.name).contains("build.gradle")'
```

will show the directories that have a file named "build.gradle".
