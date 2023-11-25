# Comments

Comments are ignored by the compiler and are used to document the code.

## Single-line comments

Single-line comments start with `//` and end at the end of the line.

```ruda
// This is a single-line comment
```

## Multi-line comments

Multi-line comments start with `/*` and end with `*/`.

```ruda
/*
This is a multi-line comment
*/
```

## Why use comments?

Comments are used to document the code. They are ignored by the compiler and are not executed. They are used to explain the code to other programmers.

example:

```ruda
// This program prints "Hello world" to the console

import "std.io"

fun main() {
    io.println("Hello world")
}
```

Compare this snippet from Hello world:

```ruda
import "std.io"

fun main() {
    io.println("Hello world")
}
```

They both do the same thing, but the commented one is much easier to understand.