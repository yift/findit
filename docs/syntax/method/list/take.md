# Take list method

The `take`  method is used to take a few items from the beginning of a list to create a new sub list.
It takes a single numeric argument which is the number of items in the new string.
If the list is shorter than the argument, it will return the list as is.

For example:

```bash
findit -w 'words().take(1) = ["package"]'
```

will show all the files that start with the word "package".
