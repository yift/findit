# Bitwise or (`|`) numeric operator

The Bitwise or (`|`) numeric operator is used to set 1 to any bit in which any operands bits are one.

For example:

```bash
findit -w '(size | 0xFF) = 0xFF'
```

Will show all the files smaller that 256 bits.
