# Is some (`IS [NOT] SOME`) operator

The is some (`IS SOME`) and is not some (`IS NOT SOME`) are used to verify that an operand is not empty.

For example:

```bash
findit -w 'content IS SOME'
```

Will show all the files that have readable content.
