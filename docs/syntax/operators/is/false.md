# Is false (`IS [NOT] FALSE`) operator
The is false (`IS FALSE`) and is not false (`IS NOT FALSE`) are used to verify that a Boolean operand is false (or not).
Note that empty `IS FALSE` will return false and empty `IS NOT FALSE` will return true.


For example:
```bash
findit -w 'content.contains("test") IS FALSE'
```
Will show all the files that have not `test` in their content.

