# Any list method
The `any`  method is used to check if a single one of the items of a list pass a condition.

The method takes a single argument that is the function of the condition. The format is `<list>.any($<name> <condition_with_$name>)`.

For example:
```bash
findit  -w 'files.any($file $file.size > 2048)'
```
will show only the directories in which there is a file that is larger than 2048 bytes.

