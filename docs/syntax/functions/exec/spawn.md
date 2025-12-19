# spawn function

The `spawn` (or `fire`) function will execute an external process and will return the process ID without waiting for the process to complete. The first argument should be a path (or a string) to the executable to run and the others are the arguments.
You can add an `INTO files` at the end of the argument list to point the stdout to another file.

For example:

```bash
findit -w 'extension = "bash"' -d '`name` pid: `spawn(path INTO ("/tmp/outputs/" + name + ".txt"))`'
```

Will run all the bash files and output the output into the `/tmp/outputs/` directory and:

```bash
findit -w 'extension = "txt"' -d '`name` pid: `spawn(@rm, path)`' /tmp/outputs/
```

Will delete those files
