# Modules

This will be a short tutorial on how modularity works in Ruda.

## What is a module?

A module is a file that contains code. Modules can be imported into other modules. This allows you to use the code from one module in another module.

## Creating a module

To create a module, you need to create a file with the `.rd` extension. This is the extension used for Ruda modules.

Example:

```ruda
// file: math.rd

fun add(x: int, y: int): int {
    return x + y
}

const PI = 3.14
```

This creates a module named `math` that contains a function named `add` and a constant named `PI`.

## Importing a module

To import a module, you need to use the `import` keyword, followed by the name of the module.

Example:

```ruda
import "math.rd"

fun main() {
    let result = math.add(1, 2)
    io.println(result) // 3
}
```

This imports the `math` module and uses the `add` function from it.

## Aliasing a module

You can also alias a module using the `as` keyword.

Example:

```ruda
import "math.rd" as m

fun main() {
    let result = m.add(1, 2)
    io.println(result) // 3
}
```
