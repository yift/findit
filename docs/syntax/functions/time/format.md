# Format function
The `format` function is used to convert a date into a string using a specific format.

The syntax of the format function is: `format(<date> AS <format>)`
Where date is the date to format and the format is a string with Chrono available formats (see [here](http://docs.rs/chrono/latest/chrono/format/strftime/index.html))

For example:
```bash
findit -d '`name` created at: `format(created as "%A %d/%B/%Y")'
```
Will show files and their creation date in a long format.
