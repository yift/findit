# findit Cookbook

Real-world examples and common use cases for `findit`.

## Table of Contents

- [Finding Files](#finding-files)
- [Filtering by Size](#filtering-by-size)
- [Filtering by Date](#filtering-by-date)
- [Searching Content](#searching-content)
- [Working with Extensions](#working-with-extensions)
- [Complex Filters](#complex-filters)
- [Custom Output](#custom-output)
- [Working with Directories](#working-with-directories)
- [File Permissions](#file-permissions)
- [Performance Tips](#performance-tips)

---

## Finding Files

### List all files (excluding directories)

```bash
findit --where 'IS FILE'
```

### List only directories

```bash
findit --where 'IS DIR'
```

### Find files with specific name

```bash
findit --where 'name = "README.md"'
```

### Find files matching pattern (case-insensitive)

```bash
findit --where 'name.toLower().contains("test")'
```

### Find files by exact stem (name without extension)

```bash
findit --where 'stem = "index"'
```

### Find files using regex

```bash
findit --where 'name MATCHES "^test_.*\.rs$"'
```

---

## Filtering by Size

### Find files larger than 1MB

```bash
findit --where 'size > 1048576'
```

### Find files between 1KB and 1MB

```bash
findit --where 'size BETWEEN 1024 AND 1048576'
```

### Find empty files

```bash
findit --where 'IS FILE AND size = 0'
```

### Find the 10 largest files

```bash
findit --where 'IS FILE' --order-by 'size DESC' --limit 10
```

### Display file sizes in human-readable format

```bash
findit --where 'IS FILE' \
  --display '`name`: `CASE 
    WHEN size > 1073741824 THEN (size / 1073741824) AS STRING + "GB"
    WHEN size > 1048576 THEN (size / 1048576)  AS STRING + "MB"
    WHEN size > 1024 THEN (size / 1024)  AS STRING + "KB"
    ELSE size AS STRING + "B"
  END`'
```

### Find directories with total size

```bash
findit --where 'IS DIR' \
  --display '`name`: `walk().filter($f $f.IS FILE).map($f $f.size).sum()` bytes'
```

---

## Filtering by Date

### Files modified in the last 24 hours

```bash
findit --where 'modified > now() - 86400'
```

### Files modified in the last week

```bash
findit --where 'modified > now() - 604800'
```

### Files modified after a specific date

```bash
findit --where 'modified > @(2024-01-01)'
```

### Files created between two dates

```bash
findit --where 'created BETWEEN @(2024-01-01) AND @(2024-12-31)'
```

### Find oldest files

```bash
findit --where 'IS FILE' --order-by 'modified' --limit 10
```

### Find newest files

```bash
findit --where 'IS FILE' --order-by 'modified DESC' --limit 10
```

### Files modified but not in the last hour

```bash
findit --where 'modified < now() - 3600'
```

---

## Searching Content

### Files containing specific text

```bash
findit --where 'content.contains("TODO")'
```

### Files containing text (case-insensitive)

```bash
findit --where 'content.toLower().contains("todo")'
```

### Files containing multiple terms

```bash
findit --where 'content.contains("import") AND content.contains("export")'
```

### Files NOT containing text

```bash
findit --where 'IS FILE AND NOT content.contains("test")'
```

### Files with specific first line

```bash
findit --where 'content.lines().first() = "#!/bin/bash"'
```

### Files containing regex pattern

```bash
findit --where 'content MATCHES "function\s+\w+\s*\("'
```

### Count lines in files

```bash
findit --where 'IS FILE' --display '`name`: `content.lines().length()` lines'
```

### Find files with more than 100 lines

```bash
findit --where 'content.lines().length() > 100'
```

### Find files with TODO comments

```bash
findit --where 'content.lines().any($line $line.contains("// TODO"))'
```

---

## Working with Extensions

### Find all files with specific extension

```bash
findit --where 'extension = "rs"'
```

### Find multiple extensions

```bash
findit --where 'extension = "rs" OR extension = "toml"'
```

### Find files without extension

```bash
findit --where 'extension IS NONE'
```

### Case-insensitive extension search

```bash
findit --where 'extension.toLower() = "jpg"'
```

### Group files by extension

```bash
findit --where 'IS DIR' \
  --display '`name`: `files.groupBy($f $f.extension).map($g $g::key + ": " + $g::values.length())`'
```

### Count files per extension in current directory

```bash
findit --where 'IS FILE' --max-depth 1 \
  --display '`extension`: 1' | sort | uniq -c
```

---

## Complex Filters

### Rust files without tests

```bash
findit --where 'extension = "rs" AND NOT content.contains("#[cfg(test)]")'
```

### Large text files

```bash
findit --where 'extension = "txt" AND size > 1048576'
```

### Recently modified source files

```bash
findit --where 'extension = "rs" AND modified > now() - 86400'
```

### JSON files with specific content

```bash
findit --where 'extension = "json" AND content.contains("\"version\"")'
```

### Files in build directories

```bash
findit --where 'path.contains("/build/") OR path.contains("/target/")'
```

### Files NOT in build directories

```bash
findit --where 'NOT (path.contains("/build/") OR path.contains("/target/"))'
```

### Files with same name as parent directory

```bash
findit --where 'name = parent.name + "." + extension'
```

### Hidden files (starting with dot)

```bash
findit --where 'name.hasPrefix(".")'
```

### Backup files

```bash
findit --where 'name.hasSuffix("~") OR name.hasSuffix(".bak")'
```

---

## Custom Output

### Show name and size

```bash
findit --display '`name` (`size` bytes)'
```

### Show full path

```bash
findit --display '`absolute`'
```

### Show relative path with line count

```bash
findit --where 'IS FILE' --display '`path`: `content.lines().length()` lines'
```

### CSV output

```bash
findit --where 'IS FILE' --display '`name`,`size`,`modified`'
```

### JSON-like output

```bash
findit --display '{"name": "`name`", "size": `size`, "modified": "`modified`"}'
```

### Markdown table

```bash
echo "| Name | Size | Modified |"
echo "|------|------|----------|"
findit --where 'IS FILE' --display '| `name` | `size` | `modified` |'
```

### Custom delimiters for interpolation

```bash
findit --display 'File: <<name>>, Size: <<size>>' \
  --expr-start '<<' --expr-end '>>'
```

---

## Working with Directories

### Directories with more than 10 files

```bash
findit --where 'IS DIR AND files.length() > 10'
```

### Empty directories

```bash
findit --where 'IS DIR AND files.length() = 0'
```

### Directories containing specific file

```bash
findit --where 'IS DIR AND (me / "Cargo.toml").exists'
```

### Directories without README

```bash
findit --where 'IS DIR AND NOT (me / "README.md").exists'
```

### Count subdirectories

```bash
findit --where 'IS DIR' --display '`name`: `files.filter($f $f.IS DIR).length()` subdirs'
```

### Total size of directory contents

```bash
findit --where 'IS DIR' \
  --display '`name`: `files.filter($f $f.IS FILE).map($f $f.size).sum()` bytes'
```

### Directories with many text files

```bash
findit --where 'IS DIR AND walk().filter($f $f.extension = "txt").length() > 100'
```

---

## File Permissions

### Find executable files

```bash
findit --where 'NOT IS DIR AND permission & 0o111 != 0'
```

### Find world-writable files (security risk)

```bash
findit --where 'permission & 0o002 != 0'
```

### Find files owned by current user

```bash
findit --where 'owner = env("USER")'
```

### Find files NOT owned by current user

```bash
findit --where 'owner != env("USER")'
```

### Find setuid files (security audit)

```bash
findit --where 'permission & 0o4000 != 0'
```

### Find files with specific permissions

```bash
findit --where 'permission = 0o644'
```

---

## Performance Tips

### Limit depth for faster searches

```bash
findit --max-depth 3 --where 'extension = "rs"'
```

### Skip deep directories

```bash
findit --where 'depth <= 3 AND extension = "rs"'
```

### Filter before reading content

```bash
# Good - filters by extension first
findit --where 'extension = "txt" AND content.contains("TODO")'

# Bad - reads all files
findit --where 'content.contains("TODO") AND extension = "txt"'
```

### Use WITH to cache expensive operations

```bash
findit --where 'WITH $c AS content DO $c.contains("test") AND $c.contains("code") END'
```

### Limit results for exploration

```bash
findit --where 'size > 1048576' --limit 10
```

### Use depth-first for faster initial results

```bash
findit --files-first --where 'extension = "rs"' --limit 1
```

---

## Advanced Examples

### Find duplicate files by size

```bash
findit --where 'IS FILE' --order-by 'size' \
  --display '`size`:`name`' | sort -n
```

### Find duplicate files by content (first 100 chars)

```bash
findit --where 'IS FILE' \
  --display '`content.take(100)`|`path`' | sort | uniq -d
```

### List all TODO/FIXME comments

```bash
findit --where 'IS FILE AND content.contains("TODO")' \
  --display '`path`:`content.lines().enumerate().filter($l $l::value.contains("TODO")).map($l "Line " + $l::index + ": " + $l::value.trim())`'
```

### Find orphaned test files (no corresponding source)

```bash
findit --where 'stem.hasSuffix("_test") AND NOT (parent / (stem.removeSuffix("_test") + ".rs")).exists'
```

### Generate file tree with sizes

```bash
findit --where 'depth <= 2' \
  --display '`"  " * depth + name + " (" + size + ")"`'
```

### Find configuration files

```bash
findit --where 'name MATCHES "^\..*rc$" OR extension = "conf" OR extension = "cfg"'
```

### Identify potential memory leaks in C/C++ code

```bash
findit --where '(extension = "c" OR extension = "cpp") AND 
  WITH $c AS content DO 
    $c.contains("malloc") AND NOT $c.contains("free")
  END'
```

### Find shell scripts without shebang

```bash
findit --where 'extension = "sh" AND NOT content.hasPrefix("#!")'
```

### Audit file ages across project

```bash
findit --where 'IS FILE' \
  --order-by 'modified DESC' \
  --display '`modified`: `name`' \
  --limit 20
```

---

## Integration Examples

### Use with xargs to process files

```bash
findit --where 'extension = "rs"' | xargs wc -l
```

### Pipe to other commands

```bash
findit --where 'extension = "md"' | xargs grep -l "TODO"
```

### Generate ctags for all source files

```bash
findit --where 'extension = "rs" OR extension = "py"' | ctags -L -
```

### Archive matching files

```bash
findit --where 'extension = "log" AND modified < now() - 2592000' | \
  tar -czf old_logs.tar.gz -T -
```

### Delete old temporary files

```bash
findit /tmp --where 'name.hasPrefix("tmp") AND modified < now() - 86400' | \
  xargs rm
```

---

## Debugging

### Show which files are being processed

```bash
findit --where 'extension = "rs"' --debug-log=/tmp/findit.log
```

### Debug expression evaluation

```bash
findit --where 'size.debug($s "Size is: " + $s) > 1024'
```

### See depth of processed files

```bash
findit --display 'Depth `depth`: `path`'
```

---

## Tips and Tricks

1. **Quote your expressions**: Always use single quotes in the shell:

```bash
   findit --where 'size > 1024'  # Good
   findit --where "size > 1024"  # Might break with special chars
```

1. **Chain methods**: Methods can be chained for powerful expressions:

```bash
   findit --where 'content.toLower().trim().contains("todo")'
```

1. **Use variables**: WITH expressions help avoid re-computation:

```bash
   findit --where 'WITH $lines AS content.lines() DO $lines.length() > 100 AND $lines.first().contains("#!") END'
```

1. **Test incrementally**: Build complex filters step by step:

```bash
   findit --where 'extension = "rs"'
   findit --where 'extension = "rs" AND size > 1024'
   findit --where 'extension = "rs" AND size > 1024 AND NOT content.contains("test")'
```

1. **Use --limit during development**: Faster feedback while building filters:

```bash
   findit --where 'complex expression' --limit 5
```

1. **Bookmark common patterns**: Create shell aliases:

```bash
   alias findrs='findit --where "extension = \"rs\""'
   alias findbig='findit --where "size > 1048576" --order-by "size DESC"'
```

---

For more information, see:

- [Usage Guide](usage.md)
- [Syntax Reference](syntax/index.md)
- [Installation](install.md)
