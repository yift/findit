# Take string method
The `take`  method is used to take a few characters from the beginning of a string to create a new sub string.
It takes a single numeric argument which is the number of characters in the new string.
If the string is shorter than the argument, it will return the string as is.

For example:
```bash
findit -w 'content.take(7) = "package"'
```
will show all the files that start with the string "package".

