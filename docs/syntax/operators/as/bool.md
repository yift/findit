# As boolean (`As BOOLEAN`) operator

The `AS BOOLEAN`  (or `AS BOOL`) operator is used to cast an operand to a boolean.
Depending on the type of operand, the casting will work as:

| operand type | Rule |
| --- | --- |
| String | Will be true if the value is `yes`, `true,`y`, or`t` (case insensitive) |
| Boolean | Will return the value as is. |
| Date | Will return true. |
| Number | Will return true if the number is not zero (0). |
| Path | Will return true if the file exists |
| List | Will return true if the list is not empty |
| Class | Will return true if the class is not empty |
| Empty | Will return Empty |

For example:

```bash
findit -w 'content.take(1) AS BOOLEAN'
```

Will show all the files that starts with `y`, `Y`, `t`, or `T`
