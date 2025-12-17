# Parse function

The `parse` function is used to convert a string into a date using a specific format.

The syntax of the parse function is: `parse(<string> FROM <format>)`
Where date is the date to format and the format is a string with Chrono available formats (see [Chrono docs](http://docs.rs/chrono/latest/chrono/format/strftime/index.html))

For example:

```bash
findit -w 'created > parse("12/12/2025" FROM "%d/%m/%Y")'
```

Will show all the files that had been created after December 12th.
