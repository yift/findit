# Is none (`IS [NOT] NONE`) operator
The is some (`IS NONE`) and is not some (`IS NOT NONE`) are used to verify that an operand is empty.


For example:
```bash
findit -w 'content IS NONE'
```
Will show all the files that have no readable content (for example, directories).

