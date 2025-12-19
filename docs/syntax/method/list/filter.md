# Filter list method

The `filter`  method is used to filter a list and return another list with the filtered values.

The method takes a single argument that is the function to filter the item. The format is `<list>.filter($<name> <filter_with_$name>)`.

For example:

```bash
findit  -w 'files.filter($dir $dir.IS DIR).length() > 3'
```

will show only the directories that have more than 3 sub directories
