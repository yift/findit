# Class creation (`{}`)
To create a class, one can use the `{:<key_1> <value_1>, :<key_2> <value_2>, ...}` syntax, where the keys are the name of the fields and the values can be dynamic.
To access a class field, one can use the `::` operator. i.e. `$class::email` will access a key name `:email` in a value named `$class`.



For example:
```bash
findit -w 'WITH $lines AS lines(), $firsAndLast AS {:first $lines.first(), :last $lines.last()} DO $firsAndLast::first = $firsAndLast::last END'
```
Will list the files that start and end with the same line.

