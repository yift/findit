# Dates Literals

To use a date literal in an expression, use the `@(<date>)`syntax where date can be

* [RFC-3339](https://datatracker.ietf.org/doc/html/rfc3339) for example: `@(2025-10-24T00:11:22.00Z)`
* `dd/MMM/yyyy` for example: `@(24/Oct/2025)`
* `yyyy-mm-dd` for example: `@(2025-10-24)`
* `dd/MMM/yyyy hh:mm` for example: `@(24/Oct/2025 00:11)`
* `dd/MMM/yyyy hh:mm:ss` for example: `@(24/Oct/2025 00:11:22)`
* `dd/MMM/yyyy hh:mm:ss.ms` for example: `@(24/Oct/2025 00:11:22.00)`
* `yyyy-mm-dd hh:mm` for example: `@(2025-10-24 00:11)`
* `yyyy-mm-dd hh:mm:ss` for example: `@(2025-10-24 00:11:22)`
* `yyyy-mm-dd hh:mm:ss.ms` for example: `@(2025-10-24 00:11:22.00)`
* `dd/MMM/yyyy hh:mm zz` for example: `@(24/Oct/2025 00:11 +0400)`
* `dd/MMM/yyyy hh:mm:ss zz` for example: `@(24/Oct/2025 00:11:22 +0400)`
* `dd/MMM/yyyy hh:mm:ss.ms zz` for example: `@(24/Oct/2025 00:11:22.00 +0400)`
* `yyyy-mm-dd hh:mm zz` for example: `@(2025-10-24 00:11 +0400)`
* `yyyy-mm-dd hh:mm:ss zz` for example: `@(2025-10-24 00:11:22 +0400)`
* `yyyy-mm-dd hh:mm:ss.ms zz` for example: `@(2025-10-24 00:11:22.00+0400)`

For example:

```bash
findit -w 'modified < @(20/Aug/2025)'
```

will find all the files that have been modified before August 20 2025.
