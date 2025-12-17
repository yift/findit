# As Number (`AS NUMBER`) operator

The `AS NUMBER`  (or `AS NUM`, or `AS INT`, or `AS INTEGER`) operator is used to cast an operand to a number.
Depending on the type of operand, the casting will work as:

| operand type | Rule |
| --- | --- |
| String | If the string is a decimal number, will return that number, otherwise, will return empty |
| Boolean | If the operand is true, will return 1, if it's false will return 0. |
| Date | Will return the number of seconds since Unix epoch. |
| Number | Will return the value as is. |
| List | Will return the number of elements in the list |
| Class | Will return the number of fields in the class |
| Path | Will return empty |
| Empty | Will return Empty |

For example:

```bash
findit -d 'file: `path` modified at: `modified` 1 day before modification was `((modified AS NUMBER) - 1 * 60 * 60 * 24) AS DATE`'
```

Will show all the files and the day before they had been modified
