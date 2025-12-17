# Divide (`/`) numeric operator

The Divide (`/`) numeric operator is used to divide two numbers.
Note that if the right operand is zero, the result will be empty.
Note that the result will always be an integer. That is, `20/3` will be `6`.

For example:

```bash
findit -w 'size / 1024 > 2'
```

Will filter all the files with size larger than 1k.
