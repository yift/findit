# IndexOf list method
The `indexOf` (or `index_of`)  method is used to check the location of an item within a list.
It accept a single argument which is the item to check for.

If the item is not contains within the list the result will be empty.

For example:
```bash
findit -w 'lines().indexOf("[package]") = 0'
```
will show the files that start with `[package]`.

