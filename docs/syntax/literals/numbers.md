# Numeric Literals
`findit` support only unsigned 64bits integers (0-18,446,744,073,709,551,615). To write an number, one can simply use the decimal representation of the number. For example:
```bash
findit -d '`size / 1024`'
```
Will display the size of the files divided by 1024.


One can also use octal representation using the `0o` prefix. For example:
```bash
findit -w 'NOT IS DIR AND permissions & 0o111 != 0'
```
will display only the executable files in the directory.


One can also use hexadecimal representation using the `0x` prefix or binary representation using the `0b` prefix. For example:
```bash
findit -d '`size / 0x1000`'
```
Will display the size of the files divided by 4096.
