# findit

`findit` is a simple and powerful command line utility that can be used to search for files in a directory hierarchy as an alternative to `find`.

## What can you use it for

You can use this tool to find files within a directory using complex filters over the file properties, content or structure.

For example:

```bash
findit src --where 'extension == "rs" AND not content.contains("#[cfg(test)]")'
```

Will find all the rust files under src without any test code.

## Quick Examples

### Find large files

```bash
findit --where 'size > 1000000' --order-by 'size DESC' --limit 10
```

### Find recent files

```bash
findit --where 'modified > now() - 86400'
```

### Find files by content

```bash
findit --where 'content.contains("TODO")'
```

### Find executable files

```bash
findit --where 'NOT IS DIR AND permissions & 0o111 != 0'
```

## Documentation

- [Quick Start Guide](docs/quick-start.md)
- [Installation](docs/install.md)
- [Usage Guide](docs/usage.md)
- [Syntax Reference](docs/syntax/index.md)
- [Cookbook - Real-world Examples](docs/cookbook.md)
- Quick syntax help: `findit --help-syntax`
