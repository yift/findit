# Dot (`.`) access operator

The Dot (`.`) access operator is used to access properties of a file (see [access docs](../../access.md)) or methods of any operand. See the [Of](of.md) for more natural syntax.

For example:

```bash
findit -w 'me.name = parent.name'
```

Will display only the files with the same name as their parent. Note that `me.name` is equivalent to `name`.
