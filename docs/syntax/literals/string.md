# String Literals
To use a string literal in an expression, use double quotes. i.e.: `"<text>"`. For example:
```bash
findit -w 'extension = "jar"'
```
## Escape characters
`findit` double quoted string supports the following escapes:
* `\\` -> `\`
* `\"` -> `"`
* `\n` -> new line
* `\r` -> carriage return
* `\t` -> tab
* `\uXXXX` where `XXXX` are 0-9A-F -> unicode character `XXXX`

For example:
```bash
findit -w 'content.contains("\u03B1")'
```
Will find all the files that contains the letter `α`.
