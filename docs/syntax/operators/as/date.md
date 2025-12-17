# As Date (`AS DATE`) operator
The `AS DATE`  (or `AS TIME`, or `AS TIMESTAMP`) operator is used to cast an operand to a date.

If the operand is a string, it will try to parse the string as a date. See [here](../../literals/dates.md).
If the operand is a number, it will assume that the number is the number of seconds since Unix epoch.
If the operand is a path, the date will be the timestamp in which the file was last accessed.
Anything else will be empty.


For example:
```bash
findit -d 'file: `path` modified at: `modified` 1 day before modification was `((modified AS NUMBER) - 1 * 60 * 60 * 24) AS DATE`'
```
Will show all the files and the day before they had been modified
