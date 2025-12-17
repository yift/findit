# execute function

The `execute` (or `exec`) function will execute an external process, will wait for it to complete and will return true if it was successful. The first argument should be a path (or a string) to the executable to run and the others are the arguments.
One can add an `INTO files` at the end of the argument list to point the stdout to another file.

For example:

```bash
findit -w 'extension = "bash"' -d '`name` pid: `execute(path INTO ("/tmp/outputs/" + name + ".txt"))`'
```

Will run all the bash files and output the output into the `/tmp/outputs/` directory and:

```bash
findit -w 'extension = "txt"' -d '`name` pid: `execute(@rm, path)`' /tmp/outputs/
```

Will delete those files
