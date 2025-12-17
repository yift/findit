# DistinctBy list method
The `distinctBy` (also `uniqueBy`, `distinct_by`, or `unique_by`) method is used to remove duplicates from a list based on an argument.

The method takes a single argument that is the function to decides the file distinction. The format is `<list>.distinctBy($<name> <action_with_$name>)`.

For example:
```bash
findit  -d 'file: `me` has `files.distinctBy($file $file.extension).length()` extensions in `files.length()` files' -w 'IS DIR'
```
will show all the directories with the number of files and extensions in each directory



