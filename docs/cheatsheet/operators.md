# Operators

Here is a list of all the operators in the language.

## Arithmetic Operators

| Operator | Description | Example |
| --- | --- | --- |
| `+` | Adds two values | `5 + 5` = `10` |
| `-` | Subtracts two values | `5 - 5` = `0` |
| `*` | Multiplies two values | `5 * 5` = `25` |
| `/` | Divides two values | `5 / 5` = `1` |
| `%` | Returns the remainder of two values | `7 % 5` = `2` |

## Comparison Operators

| Operator | Description | Example |
| --- | --- | --- |
| `==` | Returns true if two values are equal | `5 == 5` = `true` |
| `!=` | Returns true if two values are not equal | `5 != 5` = `false` |
| `<` | Returns true if the first value is less than the second value | `5 < 6` = `true` |
| `>` | Returns true if the first value is greater than the second value | `5 > 6` = `false` |
| `<=` | Returns true if the first value is less than or equal to the second value | `5 <= 6` = `true` |
| `>=` | Returns true if the first value is greater than or equal to the second value | `5 >= 6` = `false` |

## Logical Operators

| Operator | Description | Example |
| --- | --- | --- |
| `&&` | Returns true if both values are true | `true && false` = `false` |
| `||` | Returns true if either value is true | `true || false` = `true` |
| `!` | Returns true if the value is false | `!true` = `false` |

## Error Handling Operators

> Note: Also have a look at [Bang](/tutorial/error-handling/#bang-operator).

| Operator | Description | Example |
| --- | --- | --- |
| `?` | Returns whether or not an expression is null | `x?` |
| `!` | Forwards an error that occurs in a function | `foo()!` |

## Bitwise Operators

> Not implemented yet.

## Assignment Operators

| Operator | Description | Example |
| --- | --- | --- |
| `=` | Assigns a value to a variable | `x = 5` |
| `+=` | Adds a value to a variable | `x += 5` |
| `-=` | Subtracts a value from a variable | `x -= 5` |
| `*=` | Multiplies a variable by a value | `x *= 5` |
| `/=` | Divides a variable by a value | `x /= 5` |

## Other Operators

| Operator | Description | Example |
| --- | --- | --- |
| `.` | Accesses a member of a struct | `foo.bar` |
| `[]` | Accesses an element of an array | `foo[0]` |
| `()` | Calls a function | `foo()` |
| `:` | Declares a type | `let x: int = 5` |
| `;` | Ends a statement | `let x = 5;` |

# Order of Operations

Order of operations is the order in which operators are evaluated. For example, in the expression `5 + 5 * 5`, the multiplication operator (`*`) is evaluated before the addition operator (`+`). This means that the expression is evaluated as `5 + (5 * 5)`, which equals `30`.

The order of operations is as follows:

1. Parentheses: `()`
2. Unary operators: `!`, `-`, `new`
3. Multiplication and division: `*`, `/`, `%`
4. Addition and subtraction: `+`, `-`
5. Comparison operators: `==`, `!=`, `<`, `>`, `<=`, `>=`
6. Logical operators: `&&`, `||`