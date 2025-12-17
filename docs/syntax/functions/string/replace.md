# Replace function
The `replace` function is used to replace substring from a string.

The replace function support two syntaxes:
## Replace raw string
To replace all the occurrences of a specific substring within a string, use: `replace(<source> from <sub_string> to <new_string>)`
Where all arguments are strings. 

## Replace regular expressions
To replace all the matches of a regular expression within a string, use: `replace(<source> pattern <pattern> to <new_string>)`
Where all arguments are strings. 
The regular expression syntax follow rust regex - see details in [here](https://docs.rs/regex/latest/regex/#syntax).
One can also refer to the groups in the regular expression using a `$n` syntax.


For example:
```bash
findit -w 'content matches "[0-9]+"' -d '`path` -> `replace(content pattern "[0-9]+" to "<$1>")`'
```
Will add `<` and `>` around all the numbers.
