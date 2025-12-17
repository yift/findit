# Of (`OF`) access operator

The Dot (`OF`) access operator is used to access properties of a file (see [access docs](../../access.md))  or methods of any operand by putting the property/method before the file or operand. See the [Dot](dot.md) for more code like syntax.

For example:

```bash
findit -w 'name = (name OF parent)'
```

Will display only the with the same name as their parent.

```bash
findit -w '(length() OF me) > 1024'
```

Will display only the large files.
