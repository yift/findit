# Quick Start Guide

Get started with `findit` in 5 minutes.

## Installation

```bash
cargo install findit-cli
```

## Basic Usage

List all files:

```bash
findit
```

List files in a directory:

```bash
findit /path/to/dir
```

## Filtering

Find all Rust files:

```bash
findit --where 'extension = "rs"'
```

Find large files:

```bash
findit --where 'size > 1000000'
```

Find recent files:

```bash
findit --where 'modified > now() - 86400'
```

## Combining Conditions

Use AND/OR to combine conditions:

```bash
findit --where 'extension = "txt" AND size > 1024'
```

## Ordering Results

Show largest files first:

```bash
findit --where 'IS FILE' --order-by 'size DESC' --limit 10
```

## Custom Output

Show file names and sizes:

```bash
findit --display 'File: `name`, Size: `size` bytes'
```

## Next Steps

- Read the [full syntax reference](syntax/index.md)
- Check out the [cookbook](cookbook.md) for more examples
- Learn about [advanced features](usage.md)
