# Repeat (`*`) string operator

The Repeat (`*`) string operator is used to repeat a string a few times. The right operand of the operator should be a number that represent the number of times the string should repeat.

For example:

```bash
findit -d '`"|" + "-" * (depth-1) * 2 + "> " + name`'
```

Will display all the files in a tree like form.
