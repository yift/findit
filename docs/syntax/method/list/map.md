# Map list method
The `map`  method is used to transform all the items in a list to another type.

The method takes a single argument that is the function to transform the items. The format is `<list>.map($<name> <action_with_$name>)`.

For example:
```bash
findit -d 'name: `name` files: `files.map($f $f.name +" has size: "+ $f.size +"b")`' -w 'IS DIR'
```
will show the directories with their names and sizes

