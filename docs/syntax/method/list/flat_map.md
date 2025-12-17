# FlatMap list method
The `flatMap` (or `flat_map`)  method is used to transforms each element into a list and concatenates all resulting lists into a single flat list.

The method takes a single argument that is the function to transform the items to a list. The format is `<list>.flatMap($<name> <action_with_$name>)`.

For example:
```bash
findit -w 'files.flatMap($file $file.files).map($file $file.name).contains("build.gradle")'
```
will show the directories that have a file name `build.gradle` as a grandchild.

