# Skip string method
The `skip`  method is used to skip a few characters from the beginning of a string to create a new sub string.
It takes a single numeric argument which is the number of characters to skip.
If the string is shorter than the argument, it will return an empty string.

For example:
```bash
findit -w 'content.skip(2).take(5) = "ckage"'
```
will show all the files that start with the string "??ckage".

