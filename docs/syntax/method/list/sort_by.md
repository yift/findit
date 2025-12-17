# SortBy list method

The `sortBy` (also `orderBy`, `sort_by`, or `order_by`) method is used to sort the arguments in a list by a specific function.

The method takes a single argument that is the function to sort the items by. The format is `<list>.sortBy($<name> <action_with_$name>)`.

For example:

```bash
findit  -d 'name: `name` first 5 files: `files.sortBy($file $file.size).map($file $file.name).take(5)`' -w 'IS DIR'
```

will show all the directories with the first five files order by size.
