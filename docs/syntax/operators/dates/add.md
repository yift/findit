# Add (`+`) date operator
The Add (`+`) date operator is used to add seconds to a date.


For example:
```bash
findit -w 'modified  + 3600 > now()'
```
Will show only the files that had been modified in the last hour.

