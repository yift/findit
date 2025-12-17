# All list method

The `all`  method is used to check if all the items of a list pass a condition.

The method takes a single argument that is the function of the condition. The format is `<list>.all($<name> <condition_with_$name>)`.

For example:

```bash
findit  -w 'files.all($file $file.size > 2048) AND files.length() > 0'
```

will show only the directories in which all the files are larger than 2048 bytes.
