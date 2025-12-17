# GroupBy list method
The `groupBy` (or `group_by`) method is used to group the items in a list to a list of groups.

The method takes a single argument that is the function of the condition. The format is `<list>.groupBy($<name> <action_with_$name>)`.

The returned list of items will be a class with `:key` as the result of the action and `:values` as the list of items that answer that value.

For example:
```bash
findit  -w 'IS DIR' -d 'name: `name` - `files.groupBy($file $file.extension).map($group {:extension $group::key, :count $group::values.length()})`'
```
will list the directory with the number of files per extensions.

