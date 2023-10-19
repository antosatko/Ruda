# Functions

Functions are useful for writing code that you want to reuse. They can be defined anywhere in the file, and can be called anywhere in the file. You already know how to use functions, because `main` is a function.

## Defining a function

To define a function, use the `fun` keyword. The body of the function is enclosed in curly braces.

```ruda
fun greet() {
    io.println("Hello world")
}
```

This defines a function called `greet`. Now we can call the function.

```ruda
import "std.io"

fun greet() {
    io.println("Hello world")
}

fun main() {
    greet()
    greet() // We can call the function multiple times
    greet()
    greet()
}
```

This will print `Hello world` to the console 4 times.

## Parameters

Functions can take parameters. Parameters are variables that are passed to the function when it is called. They are used to pass data to the function.

```ruda
fun greet(name: string) {
    io.println("Hello " + name)
}

fun main() {
    greet("Terry")
}
```

This will print `Hello Terry` to the console.

Parameters need to have a type.

## Return values

Functions can return values. Return values are used to pass data from the function to the caller.

```ruda
fun add(a: int, b: int): int {
    return a + b
}

fun main() {
    let result = add(5, 10)
    io.println(result)
}
```

This will print `15` to the console.

Functions can only return one value which can be of any type.


## Anonymous functions

Functions can be defined without a name. These are called anonymous functions.

```ruda
import "std.io"

fun main() {
    let greet = fun(name: string) {
        io.println("Hello " + name)
    }

    greet("Terry")
}
```

The type of `greet` is `fun(string)`.

You can write the same thing like this `let greet: fun(string) = fun(name: string) { ... }`.