# Find it language syntax

## File properties

One can access different file properties like `name`, `extension`, `size` and so on: See details in [access docs](access.md).

## Literal values

Literal values, i.e. Numbers, String, Booleans... can be written as is. For example, `2` represent the number two. See more details in:

* [Numbers](literals/numbers.md)
* [Booleans](literals/bools.md)
* [Strings](literals/string.md)
* [Paths](literals/paths.md)
* [Dates](literals/dates.md)

## Binary operators

Binary operators are operators that appears between two expressions and apply to both of them. For example the plus (`+`) binary operator can be used to add two number like:  `2 + 3` (which is two plus 3). See more details in:

### Comparison operators

* [Equals (`=` or `==`)](operators/compare/eq.md)
* [Not Equals (`<>` or `!=`)](operators/compare/neq.md)
* [Bigger than (`>`)](operators/compare/gt.md)
* [Smaller than (`<`)](operators/compare/lt.md)
* [Bigger than equals (`>=`)](operators/compare/gte.md)
* [Smaller than equals (`<=`)](operators/compare/lte.md)

### Access operators

* [Dot (`.`)](operators/access/dot.md)
* [Of (`OF`)](operators/access/of.md)

### Numeric operators - arithmetic operators that apply on numeric operands

* [Plus (`+`)](operators/numbers/plus.md)
* [Minus (`-`)](operators/numbers/minus.md)
* [Times (`*`)](operators/numbers/times.md)
* [Divide (`/`)](operators/numbers/divide.md)
* [Modulo (`%`)](operators/numbers/modulo.md)
* [Bitwise and (`&`)](operators/numbers/band.md)
* [Bitwise or (`|`)](operators/numbers/bor.md)
* [Bitwise xor (`^`)](operators/numbers/bxor.md)

### String operators

* [Concat (`+`)](operators/strings/concat.md)
* [Matches (`MATCHES`)](operators/strings/matches.md)

### Date operators

* [Add (`+`)](operators/dates/add.md)
* [Take away (`-`)](operators/dates/take-away.md)

### Path operators

* [Sub (`/`)](operators/paths/sub.md)

### Boolean logical operators

* [And (`AND`)](operators/booleans/and.md)
* [Or (`OR`)](operators/booleans/or.md)
* [Xor (`XOR`)](operators/booleans/xor.md)

## Unary operators

Unary operators are operators that appears before or after an expression. For example, the negate (`NOT`) operator will negate a Boolean operand.

### Postfix unary operators

* [Negate (`NOT`)](operators/booleans/not.md)
* [Sub (`/`)](operators/paths/sub.md)

### Prefix unary operators

#### Is

The `IS` operators is used to verify that an operand is some types. The available operators are:

* [`Is [not] true`](operators/is/true.md)
* [`Is [not] false`](operators/is/false.md)
* [`Is [not] some`](operators/is/some.md)
* [`Is [not] none`](operators/is/none.md)

#### As

The `AS` operators is used to cast an operand to another type. The available castings are:

* [`As bool`](operators/as/bool.md)
* [`As string`](operators/as/string.md)
* [`As number`](operators/as/number.md)
* [`As date`](operators/as/date.md)
* [`As Path`](operators/as/path.md)

## Parentheses

Parentheses  `(...)` are used to wrap an expression in order to force it's priority. That is, while ` 3 * 2 + 4 ` will be equals to 10, one can use `3 * (2 + 4)` which will be equals to 18.

## Expressions

### If expression

One can use an `IF` expression to choose between two cases. See details in [If docs](expressions/if.md).

### Case expression

One can use a `CASE` expression to choose between more than two cases. See details in [Case docs](expressions/case.md).

### Between

One can use a `BETWEEN` expression to filter between two values. See details in [Between docs](expressions/between.md).

### With

One can use a `WITH` expression to reuse some values. See details in [With docs](expressions/with.md).

### List

One can create a list value using the `[]` syntax, for example: `[10, 11]` will be a list that contains 10 and 11. See details in  [List docs](expressions/list.md).

### Class

One can create a class value using the `{}` syntax, for example: `{:name "John", :age 61}` will be a class with two fields, `:name` a string
with "John" and `:age` a number with 61. To access the `:age` filed, use `{:name "John", :age 61}::age`. See details in  [Class docs](expressions/class.md).

## Functions

Functions can be used to invoke a function. For example, `now()` to give the current time or `rand()` to produce a random number.

### environment functions

* [coalesce](functions/env/coalesce.md)
* [env](functions/env/env.md)
* [rand](functions/env/rand.md)

### time functions

* [now](functions/time/now.md)
* [format](functions/time/format.md)
* [parse](functions/time/parse.md)

### external process execution functions

* [execOut](functions/exec/exec_out.md)
* [spawn](functions/exec/spawn.md)
* [execute](functions/exec/execute.md)

### string functions

* [replace](functions/string/replace.md)

## Methods

Methods are functions over a specific operand. They can be invoke using the [dot](operators/access/dot.md) (or [of](operators/access/of.md)) operator. They can also be
used without an operand to refer to the current file (i.e. `lines()` is the same as `me.lines()`). Methods without any arguments can be used without the open and closed parenthesis. That
is, one can use `me.lines.length` instead of `me.lines().length()`.


### String methods

* [length](method/string/length.md)
* [toUpper](method/string/upper.md)
* [toLower](method/string/lower.md)
* [trim](method/string/trim.md)
* [trimHead](method/string/trim_head.md)
* [trimTail](method/string/trim_tail.md)
* [reverse](method/string/reverse.md)
* [take](method/string/take.md)
* [skip](method/string/skip.md)
* [split](method/string/split.md)
* [lines](method/string/lines.md)
* [words](method/string/words.md)
* [contains](method/string/contains.md)
* [indexOf](method/string/index_of.md)
* [hasPrefix](method/string/has_prefix.md)
* [hasSuffix}](method/string/has_suffix.md)
* [removePrefix](method/string/remove_prefix.md)
* [removeSuffix](method/string/remove_suffix.md)

### List methods

* [length](method/list/length.md)
* [reverse](method/list/reverse.md)
* [map](method/list/map.md)
* [filter](method/list/filter.md)
* [sum](method/list/sum.md)
* [max](method/list/max.md)
* [min](method/list/min.md)
* [avg](method/list/avg.md)
* [sort](method/list/sort.md)
* [sortBy](method/list/sort_by.md)
* [distinct](method/list/distinct.md)
* [distinctBy](method/list/distinct_by.md)
* [take](method/list/take.md)
* [skip](method/list/skip.md)
* [join](method/list/join.md)
* [first](method/list/first.md)
* [last](method/list/last.md)
* [contains](method/list/contains.md)
* [indexOf](method/list/index_of.md)
* [flatMap](method/list/flat_map.md)
* [all](method/list/all.md)
* [any](method/list/any.md)
* [groupBy](method/list/group_by.md)
* [enumerate](method/list/enumerate.md)

### Path methods

* [length](method/path/length.md)
* [lines](method/path/lines.md)
* [words](method/path/words.md)
* [walk](method/path/walk.md)
