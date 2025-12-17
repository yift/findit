# findit
`findit` is a simple and powerful command line utility that can be used to search for files in a directory hierarchy as an alternative to `find`.

## What can you use it for
One can use this tool to find files within a directory using complex filters over the file properties, content or structure.

For example:
```bash
findit src --where 'extension == "rs" AND not content.contains("#[cfg(test)]")'
```
Will find all the rust files under src without any test code.

## Installation
See installation instructions [here](docs/install.md).


## User manuel
See more details on how to use `findit` [here](docs/usage.md).

