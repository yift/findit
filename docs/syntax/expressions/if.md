# If expression

An if expression can be used to choose between two branches.
The syntax to use an if expression is:

```sql
IF <condition> THEN <true_branch> [ELSE <false_branch>] END
```

where `condition` is a Boolean expression, when true the result will be the `true_branch` when false the result will be the `false_branch`. If the `false_branch` is missing and the condition is false, the result will be empty.

For example:

```bash
findit -d 'File: `name` is a `IF size > 1024 THEN "large" ELSE "small" END` file' -w 'IS FILE'
```

Will list the files names and if they are large or small...
