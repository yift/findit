# And (`AND`) Boolean logical operator
The And (`AND`) Boolean logical operator is used check if both operands are true.


For example:
```bash
findit -w 'extension == "txt" AND lines().len() > 3'
```
Will show only the files that have more than 3 lines and the extension `txt`

