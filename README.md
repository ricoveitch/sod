# Sod

An alternative to shell script.

## Table of contents

- [Data Types](#data-types)
  - [Ranges](#ranges)
  - [Lists](#lists)
- [Conditionals](#conditionals)
- [Functions](#functions)
- [For Loops](#for-loops)
- [Comments](#comments)
- [Shell Commands](#shell-commands)
- [TODO](#todo)

## Data Types

```
1.2                     # number
s = 'bar'               # string
"foo $s"                # template string (resolves to "foo bar")
t = true || false       # boolean
[1, "hello", false]     # lists
empty = none            # none (no value)
1..5                    # range
```

### Ranges

start..end..increment

Start is inclusive, end is non-inclusive, increment is optional.

```
# start..end..increment
0..3     # 0 1 2
0..5..2  # 0 2 4
3..0     # 3 2 1
```

### Lists

Dynamic arrays

```
list = [1, "2"]
list[0] # 1
```

#### Member functions

| Name                | Notes                                | Returns                                    |
| ------------------- | ------------------------------------ | ------------------------------------------ |
| len                 | length of list                       | number                                     |
| pop                 | removes the last item                | the item that was removed or none if empty |
| push(item)          | adds to end of list                  | the new length of the list                 |
| remove(index)       | removes the item at the index        | the item that was removed                  |
| contains(item)      | checks to see if item exists in list | true if item exists else false             |
| insert(index, item) | inserts item at index                | none                                       |

### Strings

#### Literals

```
name = 'John'
full_name = name + ' Doe'
```

#### Templates

`'$'` can be used before the variable name to interpolate the value. Currently more complex expressions such as `${list.len()}` are not supported.

```
name = 'John'
full_name = "$name Doe"
```

#### Member functions

| Name                  | Notes                                              | Returns                                         |
| --------------------- | -------------------------------------------------- | ----------------------------------------------- |
| len                   | length of string                                   | number                                          |
| pop                   | removes the last character                         | the character that was removed or none if empty |
| push(string)          | adds to end of string                              | the new length of the string                    |
| remove(index)         | removes the character at the index                 | the character that was removed                  |
| contains(string)      | checks to see if the input string exists in string | true if string exists else false                |
| insert(index, string) | inserts a string at index                          | none                                            |
| trim                  | trims leading and trailing whitespace              | new string with the whitespace removed          |

## Conditionals

```
if x {
    ...
} else if y {
    ...
} else {
    ...
}
```

## Functions

```
func add(x, y) {
    return x + y
}

add(1, 2)
```

## For loops

```
for i in 0..3 {
    # 0,1,2
}

list = [1, 2]
for i in list {
    # 1,2
}
```

## Comments

There are only single line comments that start with a `#`.

```
# comment
echo "hello"
```

## Shell Commands

Shell commands are run "as is", with the exception of `$`, which will look for variables declared in the script. The output of a command may be assigned to variables as strings.

```
file = "some/path/to/file"
contents = cat $file
```

### Quirks

Setting environment variables for a command like this is not yet supported

```
FOO=BAR cmd
```

Work around is to use the `env` command instead

```
env "FOO=BAR" cmd
```

## TODO

- better error messages
- general fixes (more testing)
- SHIFT + ENTER enters a new line in interpreter
- dedicated print function
- expression inside template string
- command enhancements
