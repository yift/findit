# With expression

A with expression can be used to save some values for future use. For example, the `content` will read the content the file, which might be large, so one might keep the value in order to read it only once.
This can also help to reduce code duplications.

The syntax to use a with expression is:
with $one 1, $two $one + $one, $one \"one\" do $one + $two as string end

```sql
WITH $<name> [AS] <value> [, $<name_2> AS <value_2>...] DO <expression> END
```

where `name` (and `name_2` and so on) is the name of the value, when used in the `<expression>`, use it with the dollar prefix.

For example:

```bash
findit -w 'WITH $content as content DO $content.contains("test") AND $content.contains("code") END'
```

Will list the files that has both "code" and "test" in them.

```bash
findit -w 'WITH $content as content, $lastLine AS $content.lines().last() DO $content.contains("test") AND  NOT $lastLine.contains("test") END'
```

Will list all the files that have test in them but not in the last line.
