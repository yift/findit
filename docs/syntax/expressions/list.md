# List creation (`[]`)

To create a list, one can use the `[<item>, <item>, ...]` syntax, where the items of the list must be the same type. The items can be dynamic.

For example:

```bash
findit -w 'words().take(3) = ["one", words().skip(1).first(), "three"]'
```

Will list the files that start with "one `<something>` three".
