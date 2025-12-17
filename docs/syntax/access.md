# File properties access
One can access different properties of a file (like `name`, `size` or `content`) by using the property name directly (`extension = "txt"`), using the dot operator (`parent.name = "build"`) or using the of operator (`size of me > 1024`).
By default the property apply to the current file, but it can also apply to any expression that return a path.

The list of properties are:
* `parent` - Return the path of the parent of the file.
* `name` - Return the name of the file (with extension, without parent path).
* `path` - Return the path of the file as a string.
* `extension` - Return the file extension.
* `absolute` - Return the file absolute path.
* `me` - Return the current file (useful for the `/` operator - see [here](operators/paths/sub.md)). Can be replaced with `this` or `self`.
* `content` Return the file content as a string. If the file can not be read (not exists, a directory, no read permission...) or the content is not a string (UTF8), return empty value.
* `depth` Return the file depth (from the `findit` point of view).
* `size` - Return the file size (in bytes). Return empty for directories.
* `count` - Return the number of files under the directory or 1 if the file is not a directory.
* `created` - Return the date in which the file was created.
* `modified` - Return the date in which the file was last modified.
* `exists` - Return true if the file exists (recall one can use something like `(me/"build.gradle").exists`).
* `owner` - Return the username of the file owner.
* `group` - Return the name of the group that own the file.
* `permission` - Return the file permissions.
* `files` - If the file is a directory, return the list of files it has.
* `is dir` - true if the file is a directory.
* `is not dir` - true if the file is not a directory.
* `is file` - true if the file is a file.
* `is not file` - true if the file is not a file.
* `is link` - true if the file is a link.
* `is not link` - true if the file is not a link.


For example:
```bash
findit -w 'extension = "json"'
```
Will show all the files that have a `json` extension.


```bash
findit -w '(parent / "build").is dir'
```
Will show all the directories that have a sibling directory named `"build"`.
