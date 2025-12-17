# Skip list method
The `skip`  method is used to skip a few items from the beginning of a list to create a new sub list.
It takes a single numeric argument which is the number of items to skip.
If the list is shorter than the argument, it will return an empty list.

For example:
```bash
findit -w 'words().skip(1).take(1) = ["io"]'
```
will show all the files that the second words in them is "io".

