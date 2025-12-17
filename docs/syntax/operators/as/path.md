# As Path (`AS PATH`) operator

The `AS PATH`  (or `AS FILE`, or `AS DIR`) operator is used to cast a string operand to a date.

For example:

```bash
findit -w '(content.lines().first() AS PATH).exists'
```

Will show all the files in which the first line is a name of a file that exists
