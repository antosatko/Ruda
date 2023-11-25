# Constants

Constants are values that cannot be changed. They are declared using the `const` keyword.

Unlike variables, constants must be assigned a value when they are declared. And they can be Accessed anywhere if imported.

```ruda
const PI = 3.14
```

This declares a constant named `PI` and assigns it the value `3.14`.

## Naming conventions

Constants should be named using `UPPER_SNAKE_CASE`.

```ruda
const FAVORITE_NUMBER = 34
```

## Why use constants?

Constants are used to store values that will not change. They are useful for storing values that are used multiple times in the program.

By using constants, you can also give names to values that would otherwise be hard to understand.

When you see `WINDOW_WIDTH` in your code, you know that it is the width of the window. But if you see `800`, you have no idea what it means.

Another advantage of constants is that modules can export them. This allows other modules to use them.