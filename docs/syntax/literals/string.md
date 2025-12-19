# String Literals

To use a string literal in an expression, use double quotes. i.e.: `"<text>"`. For example:

```bash
findit -w 'extension = "jar"'
```

*Note:** String literals are case-sensitive. `"jar"` is not equal to `"JAR"`. The expression language itself (keywords, properties, methods) is case-insensitive, but the string values you compare against are not.

## Case-Insensitive String Comparison

To perform case-insensitive comparisons, convert strings to the same case first:

```bash
# Case-insensitive extension check
findit -w 'extension.toLower() = "jar"'

# Case-insensitive content search
findit -w 'content.toLower().contains("todo")'
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
