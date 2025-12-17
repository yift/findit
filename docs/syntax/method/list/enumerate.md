# Enumerate list method
The `enumerate` method is used to map each item of a list to a list of the items and their indices.

The returned list of items will be a class with `:index` as the index of the item and `:item` as the item.

For example:
```bash
findit  -w 'IS DIR' -d 'name: `name` - `files.map($file $file.name).sort().enumerate().filter($i $i::index % 4 ==0).map($i $i::item)`'
```
will list the directory with the 1/4 of the files.

