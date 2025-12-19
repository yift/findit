# File properties access

You can access different properties of a file (like `name`, `size` or `content`) by using the property name directly (`extension = "txt"`), using the dot operator (`parent.name = "build"`) or using the of operator (`size of me > 1024`).
By default the property apply to the current file, but it can also apply to any expression that return a path.

The list of properties are:

* `parent` - The path of the parent of the file.
* `name` - The name of the file (with extension, without parent path).
* `path` - The path of the file as a string.
* `extension` - The file extension.
* `stem` - The name of the file (without extension, without parent path).
* `absolute` - The file absolute path.
* `me` - The current file (useful for the `/` operator - see [sub operator](operators/paths/sub.md)). Aliases: `this`, `self`.
* `content` The file content as a string. If the file can not be read (not exists, a directory, no read permission...) or the content is not a string (UTF8), return empty value.
* `depth` The file depth (from the `findit` point of view).
* `size` - The file size (in bytes). Return empty for directories.
* `count` - The number of files under the directory or 1 if the file is not a directory.
* `created` - The date in which the file was created.
* `modified` - The date in which the file was last modified.
* `exists` - `true` if the file exists (recall you can use something like `(me/"build.gradle").exists`).
* `owner` - The username of the file owner.
* `group` - The name of the group that own the file.
* `permission` - The file permissions.  Aliases: `permissions`.
* `files` - If the file is a directory, the list of files it has.
* `is dir` - `true` if the file is a directory.
* `is not dir` - `true` if the file is not a directory.
* `is file` - `true` if the file is a file.
* `is not file` - `true` if the file is not a file.
* `is link` - `true` if the file is a link.
* `is not link` - `true` if the file is not a link.

For example:

```bash
findit -w 'extension = "json"'
```

Will show all the files that have a `json` extension.

```bash
findit -w '(parent / "build").is dir'
```

Will show all the directories that have a sibling directory named `"build"`.

You can also use a method syntax to access the same properties. That is:

```bash
findit -w 'content().contains("import ")'
```
