# Bitwise and (`&`) numeric operator
The Bitwise and (`&`) numeric operator is used to set 1 to any bit in which both operands bits are one.


For example:
```bash
findit -w 'permissions & 0o111 != 0 AND IS FILE'
```
Will show all the executable files.

