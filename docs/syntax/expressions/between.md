# Between expression

A between expression can be used to filter when a value is between two values.
The syntax to use a between expression is:

```sql
<operand> BETWEEN <min> AND <max>
```

(which is equivalent to `<operand> >= <min> AND <operand> <= <max>`, but will compute the operand only once)

For example:

```bash
findit -w 'size BETWEEN 1024 AND 2048'
```

Will list the files whose size is between 10-24 bytes and 2048 bytes.
