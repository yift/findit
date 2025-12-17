# Case expression
A case expression can be used to choose between a few branches.
The syntax to use an case expression is:
```
CASE WHEN <condition_1> THEN <branch_1> WHEN <condition_2> THEN <branch_2> WHEN <condition_3> THEN <branch_3> ... [ELSE <default_branch>] END
```

where `condition_N` is a Boolean expression, when true the result will be the `branch_N` when no condition is true the result will be the `default_branch`. If the `default_branch` is missing and no condition is true, the result will be empty.


For example:
```bash
findit -d 'File: `name` is a `CASE WHEN extension = "txt" THEN "TEXT" WHEN extension = "json" THEN "JSON" WHEN extension = "bash" THEN "bash" ELSE extension END` file' -w 'IS FILE'
```
Will list the files names and their type...
