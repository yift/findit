# Take away (`-`) date operator

The Add (`-`) date operator is used to take away seconds to a date.

For example:

```bash
findit -w 'modified  > now() - 3600'
```

Will show only the files that had been modified in the last hour.
