# Coalesce function
The `coalesce` function will accept more than one argument and will return the first non empty of them.

For example:
```bash
findit -d 'File: `name` start with: `coalesce(content.take(20), "** Unreadable **")`'
```
Will list the files and their content or `** Unreadable **` if they are not readable...
