# Error handling

Sometimes an operation can fail. For example, if you try to open a file that doesn't exist, or if you try to divide by zero. In Ruda, you can handle these errors using the `try` keyword.

```ruda
try {
    let file = io.open("file.txt")
} catch err {
    io.println(err)
}
```

The `try` keyword is used to call a function that can fail. If the function fails, the `catch` block is executed. The `catch` block is used to handle the error.

The `catch` block takes a variable that stores the error.

## Catch statements

You can have multiple `catch` statements to handle different errors.

```ruda
try {
    let file = io.open("file.txt")
} catch err: io.FileNotFound {
    io.println(err.msg())
} catch err {
    io.println(err.msg())
}
```

## Yeet

The `yeet` keyword is used to throw an error.

```ruda
fun divide(x: int, y: int)!: int {
    if y == 0 {
        yeet Error("cannot divide by zero")
    }

    return x / y
}
```

## Error

If you dont need to throw an error but you don't care about the error type, you can use the `Error` type as shown above.

## Bang operator

If we don't want to handle the error, we can use the bang operator to ignore the error. It will be forwarded to the caller.

```ruda
fun divide(x: int, y: int)!: int {
    if y == 0 {
        yeet Error("cannot divide by zero")
    }

    return x / y
}

fun main() {
    let result = divide(10, 0)! // this will crash
}
```

## Error declaration

You can declare an error type using the `error` keyword.

```ruda
error DivisionByZeroError(number: int) {
    message: "cannot divide by zero",
    code: 1,
}
```

This can be used like this:

```ruda
fun divide(x: int, y: int)!: int {
    if y == 0 {
        yeet DivisionByZeroError(y)
    }

    return x / y
}
```

The error field code is optional. It can be used to identify the error. Default value is 1.

You can also take message as an argument.

```ruda
error Error(message: string?) {
    message: {
        if message? {
            return message
        } else {
            return "something went wrong"
        }
    }
    code: 1,
}
```