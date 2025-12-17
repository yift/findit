# Join list method
The `join`  method is used to convert a list to a string by joining each item in the list.

It takes an  optional single string argument which is the deliminator between the item of the list. If omitted, a comma (",") will be used.

For example:
```bash
findit -d 'name: `name`, files: `files.join(", ")`' -w 'IS DIR'
```
will show all the directories and their files.

