# Matches (`MATCHES`) string operator
The Matches (`MATCHES`) string operator will return true if the left string operand will match the regular expression that is in the right spring operand.
If the right spring operand is not a valid regular expression, the result will be empty.
The regular expression syntax follow rust regex - see details in [here](https://docs.rs/regex/latest/regex/#syntax).

For example:
```bash
findit -w 'name matches "^a[a-z]+\\.rs"'
```
Will display only the files with name that follow the pattern `^a[a-z]+\\.rs`.

