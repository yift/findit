# Bitwise xor (`^`) numeric operator

The Bitwise xor (`^`) numeric operator is used to set 1 to one and only one bit in which any operands bits are one.

For example:

```bash
findit -d 'file: `me` size hash: `size ^ 0xDEADBEEF`'
```

Will show all the files with hashing over the size of the file.
