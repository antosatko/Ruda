# Ruda for busy people

In this cheatsheet, you'll find a quick overview of the Ruda language. It's not meant to be a complete reference, but rather a quick reference for the most important features of the language.

```ruda
// This is a comment

/*
This is a multi-line comment
*/

// Imports
import "std.io"
import "std.math" as mt

// Constants
const PI = 3.14

// Type aliases
type int = i32

fun main() {
    // Variables
    let x = 5
    let z: int = x + 50

    // Arrays
    let arr = [1, 2, 3, 4, 5]      // [1, 2, 3, 4, 5]
    let arr: [int; 5] = [1; 5]     // [1, 1, 1, 1, 1]

    // If statements
    if x > y {
        io.println("x is greater than y")
    } else if x < y {
        io.println("x is less than y")
    } else {
        io.println("x is equal to y")
    }

    // Loops
    for i in math.range(0, 10) {
        io.println(i)

        // continue and break
        if i == 5 {
            continue
        }
        if i == 8 {
            break
        }
    } // 0, 1, 2, 3, 4, 6, 7

    // Switch
    switch x {
        0 {
            io.println("x is 0")
        },
        1 {
            io.println("x is 1")
        },
        _ {
            io.println("x is not 0 or 1")
        }
    }

    // Error handling
    let result;
    try {
        result = math.sqrt(-1)
    } catch err: math.MathError {
        io.println("math error occurred")
    } catch err {
        io.println("error occurred, we don't care what kind")
    }

    // Error declaration
    error MathError(msg: string) {
        message: msg, // optional, defaults to "something went wrong"
        code: 1 // optional, defaults to 1 
    }

    // Error throwing
    fun sqrt(x: float): float {
        if x < 0 {
            yeet math.MathError("x must be positive")
        }
        return math.sqrt(x)
    } // yes you read that right, it's called yeet

    // Pointers
    let x = 5
    let y = &x
    let z = *y
}


// Functions
fun add(x: int, y: int): int {
    return x + y
}

// Structs
struct Point {
    x: int,
    y: int
}

// Enums
enum Color {
    Red,
    Green = 10,
    Blue = 100,
}

// Traits
trait Drawable {
    fun draw()
}

// Methods
impl Point {
    constructor(x: int, y: int) {
        self.x = x
        self.y = y
    }
    fun draw(&self) {
        io.println("Drawing point at")
    }
}

// Generics
fun add<T(Math.Arithmetics)>(x: T, y: T): T {
    return x + y
}

// Implementing traits for types
impl Point trait math.Arithmetics {
    fun add(&self, other: &Point): Point {
        return Point(self.x + other.x, self.y + other.y)
    }
    /*
    In this case you would need to finish the implementation of the trait by implementing the other methods of the trait.
    */
}
```