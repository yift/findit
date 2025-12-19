# Class creation (`{}`)

To create a class, you can use the `{:<key_1> <value_1>, :<key_2> <value_2>, ...}` syntax, where the keys are the name of the fields and the values can be dynamic.
To access a class field, you can use the `::` operator. i.e. `$class::email` will access a key name `:email` in a value named `$class`.

Example:

```bash
# Simple class example
findit -d 'File: {:name name, :size size}'

# More complex: files with same first and last line
findit -w 'WITH $lines AS lines(), $fl AS {:first $lines.first(), :last $lines.last()} DO $fl::first = $fl::last END'
```
