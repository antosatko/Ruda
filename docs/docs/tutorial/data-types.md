# Data Types

Types describe the kind of data that is stored in a variable. 

## Primitive Types

Primitive types are the most basic data types.

| Type | Description | Example |
| ---- | ----------- | ------- |
| `int` | Integer | `1` & `-1` |
| `float` | Floating point number | `3.14` |
| `bool` | Boolean | `true` & `false` |
| `string` | String | `"Hello, World!"` |
| `char` | Character | `'a'` & `'\n'` |
| `uint` | Unsigned Integer | `0` |


## Composite Types

Composite types are types that are composed of other types.

| Type | Description | Example |
| ---- | ----------- | ------- |
| `array` | Array | let arr: [int] = `[1, 2, 3]` |
| `struct` | Struct | `struct Point { x: int, y: int }` |
| `enum` | Enum | `enum Code { Ok = 200, NotFound = 404 }` |

## Pointers

Pointers are used to store the address of a variable.

```ruda
let x: int = 1

let ptr: &int = &x
```

Another type of pointer is a function pointer. Function pointers are used to store the address of a function.

```ruda
fun add(x: int, y: int): int {
    return x + y
}

let ptr = add
```

This is useful for passing functions as arguments to other functions.

## Type Casting

Type casting is used to convert a value from one type to another.

```ruda
let x: int = 1

let y = x as float
```

## Type Aliases

Type aliases help abstract away the implementation details of a type.

```ruda
type FileHandle = uint
```

## Optionals

Optionals are used to represent values that may or may not exist.

```ruda
let x: int? = 1 // x == 1

let y: int? // y == null
```

Then we can use the `?` operator to see if the value exists.

```ruda
let x: int? = 1

if x? {
    // x exists
} else {
    // x does not exist
}
```
