# Variables

Variables are used to store values. Those values can be used or modified later in the program. Variables can be declared using the `let` keyword.

```ruda
let x = 5
```

This declares a variable named `x` and assigns it the value `5`.

To declare a variable without assigning it a value, you can use the `let` keyword without an expression.

```ruda
let x
```

This declares a variable named `x` without assigning it a value.

Variables can be assigned a new value using the `=` operator.

```ruda
let x = 5
x = 10
```

Knowing this, we can rewrite our hello world program to use a variable.

```ruda
import "#io"

fun main() {
    let message = "Hello world"
    io.println(message)
}
```

This program will print `Hello world` to the console.
