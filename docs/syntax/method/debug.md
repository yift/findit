# Debug method

**Aliases:** `dbg()`

The `debug`  method is used to print text into the debug file if enabled (using the `--debug-output-file` argument). If the flag is not enabled, the method will be ignored.
The method will take a single argument that is a function that accept the method target, and will print that output to the debug file.
The return value of the method is the target (regardless whether the flag is enabled).

For example:

```bash
findit -w 'extension = "txt" AND me.debug($f "looking at: " + $f).content.debug($content "content of text file is: " + $content).contains("world")' --debug-output-file /tmp/fdbg.txt
```

will show all the text files that contains the word `world`, and will print the files content to the `/tmp/fdbg.txt` file.
