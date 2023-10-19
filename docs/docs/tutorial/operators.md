# Operators

This page contains a list of all operators in Ruda. If you are new to programming, you can skip this page for now and return when you need to learn about a specific operator.

## Arithmetic Operators

Sometimes you need to perform arithmetic operations on variables. For example, you may need to increment or decrement a variable's value.

| Operator | Description | Example |
| --- | --- | --- |
| `+` | Addition | `x + y` |
| `-` | Subtraction | `x - y` |
| `*` | Multiplication | `x * y` |
| `/` | Division | `x / y` |
| `%` | Modulus | `x % y` |

example:

```ruda
let x = 5
let y = 10

let sum = x + y
let difference = x - y
// ... and so on
```

## Assignment Operators

Assignment operators are used to assign values to variables.

| Operator | Description | Example |
| --- | --- | --- |
| `=` | Assign | `x = y` |
| `+=` | Add and assign | `x += y` |
| `-=` | Subtract and assign | `x -= y` |
| `*=` | Multiply and assign | `x *= y` |
| `/=` | Divide and assign | `x /= y` |

example:

```ruda
let x = 5
let y = 10

x += y // the same as: x = x + y

// ... and so on
```

## Comparison Operators

Comparison operators are used to compare values.

| Operator | Description | Example |
| --- | --- | --- |
| `==` | Equal | `x == y` |
| `!=` | Not equal | `x != y` |
| `<` | Less than | `x < y` |
| `>` | Greater than | `x > y` |
| `<=` | Less than or equal | `x <= y` |
| `>=` | Greater than or equal | `x >= y` |

example:

```ruda
let x = 5
let y = 10

let equal = x == y // false
// ... and so on
```

## Logical Operators

Logical operators are used to combine boolean expressions.

| Operator | Description | Example |
| --- | --- | --- |
| `&&` | Logical AND | `x && y` |
| `||` | Logical OR | `x || y` |
| `!` | Logical NOT | `!x` |

# Error handling operators

Error handling operators are used to handle errors.

| Operator | Description | Example |
| --- | --- | --- |
| `?` | Exists? | `x?` returns `true` if `x` is not `null` |
| `!` | Trust me | `x()!` forwards the error if `x` threw an error |