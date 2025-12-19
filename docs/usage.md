# How to use findit

## Printing the help screen

Using:

```bash
findit --help
```

will print the help screen with all the available options to the standard output.

## Choosing the root to  start from

By default, using:

```bash
findit
```

will just print all the files and directory under the current directory. You can use another root by passing it as an argument to the command. For example:

```bash
findit ./src
```

will only list the files under `./src` while

```bash
findit /bin
```

will list all the files under `/bin`

## Filtering files

By default, `findit` will display all the files under the root directory. To filter files, you can use the `--where` (or `-w`) parameter.
For example:

```bash
findit --where 'path.contains("build") AND extension == "class"'
```

will find all the class files under any build directory in the current directory.

To see more details on the available syntax, see [syntax language docs](syntax/index.md). Please note, you can only filter based on Boolean values.

## Ordering the files

### Explicit order

By default, `findit` will display the files by the order in which it found them. You can change this by using the `--order-by` (or `-o`) parameter.
For example:

```bash
findit  --order-by 'size DESC'  -l 10 -w 'IS FILE'
```

will display the 10 largest files, while:

```bash
findit  --order-by 'size'  -l 10 -w 'IS FILE'
```

will display the 10 smallest files.

To see more details on the available syntax, see [syntax language docs](syntax/index.md)
Note that adding the `DESC` will change the order to descending order, while omitting it or adding `ASC` will make the order ascending.

You can also sort by a list of expressions. For example:

```bash
findit --order-by 'extension, size DESC'  -l 10
```

will sort the files by extension and then by the size.

### implicit order

You can also use the `--node-first` to indicate that `findit` should start from the nodes (i.e. the files and not the directories).

## Limit the depth

By default, `findit` will consider all the files in the root directory. You can change it by using the `--max-depth` (or `-x`) and/or the `--min-depth` (or `-n`) parameters. For example:

```bash
findit -n 2 -x 3
```

will only consider files with depth 2 or 3. The root depth is 0, files directly under the root have depth 1 and so on.

Note that you can also use the `depth` property of the file for more complicated filtering.

## Limit the number of results

By default, `findit` will consider all the files in the root directory. You can limit the number of results to a specific number of files by using the `--limit` (or `-l`) parameter. See example in the ordering section above.

## Controlling the output

By default, `findit` will print the path of each file that passed the filters and limitations. You can change this using the `--display` (or `-d`) parameter. The display argument syntax is text with backticks (`\``) sounding any syntax you want to display.

For example:

```bash
findit -d 'file name: `name`, size: `size`bytes, was created at `created`'
```

If the display contains any backticks (`\``), you can replace them with any other non-empty string using the`--interpolation-start` and `--interpolation-end` arguments. For example:

```bash
findit -d 'file name: <-name->, size: <-size->bytes, was created at <-created->' --interpolation-start='<-' --interpolation-end='->'
```

To see more details on the available syntax, see [syntax language docs](syntax/index.md)

## Debugging the process

You can enable debug output using the `--debug-output-file` flag to see:

- Which files `findit` considered
- Which directories it visited
- When it applied limits
- The output of the `Debug` method (see [the `Debug` method](syntax/method/debug.md))

Example:

```bash
findit --debug-output-file=/tmp/debug.log --where 'size > 1024'
```

Then view the log:

```bash
less /tmp/debug.log
```
