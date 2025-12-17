# execOut function

The `execOut` (or `executeOutput` or `execute_output` or `exec_out`) function will execute an external process and will return the stdout of that process as a
string. The first argument should be a path (or a string) to the executable to run and the others are the arguments.

For example:

```bash
findit -d '`name` has permissions `execOut(@stat, "-c", "%A", path)`'
```

Will show the permissions of each file
