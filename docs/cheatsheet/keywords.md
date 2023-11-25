# Keywords

Here is a list of all the keywords in the language.

## Variable declaration

| Keyword | Description | Example | reference |
| --- | --- | --- | --- |
| `let` | Declares a variable | `let x = 10` | [Variables](/tutorial/variables) |
| `const` | Declares a constant | `const x = 10` | [Constants](/tutorial/constants) |
| `type` | Declares a type alias | `type Foo = int` | [Type Aliases](/tutorial/data-types/#type-aliases) |

## Control flow

| Keyword | Description | Example | reference |
| --- | --- | --- | --- |
| `if` | Executes a block of code if a condition is true. | `if x == 10 { ... }` | [If Statements](/tutorial/control-flow) |
| `else` | Executes a block of code if a condition is false. | `if x == 10 { ... } else { ... }` | [Else Statements](/tutorial/control-flow/#else) |
| `else if` | Executes a block of code if a condition is false and another condition is true. | `if x == 10 { ... } else if x == 20 { ... }` | [Else If Statements](/tutorial/control-flow/#else-if) |
| `switch` | Executes a block of code based on the value of an expression. | `switch x { ... }` | [Switch Statements](/tutorial/control-flow/#switch-statements) |

## Loops

| Keyword | Description | Example | reference |
| --- | --- | --- | --- |
| `while` | Executes a block of code while a condition is true. | `while x < 10 { ... }` | [While Loops](/tutorial/control-flow/#while) |
| `for` | Executes a block of code for each element in a collection. | `for x in [1,2,3] { ... }` | [For Loops](/tutorial/control-flow/#for) |
| `loop` | Executes a block of code forever. | `loop { ... }` | [Loop Statements](/tutorial/control-flow/#loop) |
| `break` | Breaks out of a loop. | `break` | [Break Statements](/tutorial/control-flow/#break) |
| `continue` | Skips the rest of the current iteration of a loop. | `continue` | [Continue Statements](/tutorial/control-flow/#continue) |

## Functions

| Keyword | Description | Example | reference |
| --- | --- | --- | --- |
| `fun` | Declares a function. | `fun foo() { ... }` | [Functions](/tutorial/functions) |
| `return` | Returns a value from a function. | `return 10` | [Return Statements](/tutorial/functions/#return-values) |
| `overload` | Declares a function that overloads an operator. | `overload + (other: Foo): Foo { ... }` | [Operator Overloading](/tutorial/advanced/overloads) |

## Comments

| Keyword | Description | Example | reference |
| --- | --- | --- | --- |
| `//` | Single line comment | `// this is a comment` | [Comments](/tutorial/comments) |
| `/* */` | Multi line comment | `/* this is a comment */` | [Comments](/tutorial/comments) |

## Non Primitive Types

| Keyword | Description | Example | reference |
| --- | --- | --- | --- |
| `struct` | Declares a struct. | `struct Foo { a: int }` | [Structs](/tutorial/structs) |
| `enum` | Declares an enum. | `enum Foo { Right, Left }` | [Enums](/tutorial/enums) |
| `impl` | Implements methods or traits for a type. | `impl Foo { fun foo() { ... } }` | [Methods](/tutorial/methods) |
| `trait` | Declares a trait. | `trait Foo { fun foo() }` | [Traits](/tutorial/advanced/traits) |
| `import` | Imports a module. | `import "std.io"` | [Modules](/tutorial/modules) |

## Memory Management

| Keyword | Description | Example | reference |
| --- | --- | --- | --- |
| `new` | Allocates memory on the heap. | `let x = new 5` | [Memory Management](/tutorial/advanced/memory) |
| `as` | Casts a value to a different type. | `let x = 5 as float` | [Casting](/tutorial/data-types/#type-casting) |
| `?` | Checks if a value is null. | `if x? { ... }` | [Optionals](/tutorial/data-types/#optionals) |

## Error Handling

| Keyword | Description | Example | reference |
| --- | --- | --- | --- |
| `try` | Executes a block of code that can throw an error. | `try { ... }` | [Error Handling](/tutorial/error-handling) |
| `catch` | Handles an error that occured inside a try block. | `catch err { ... }` | [Catch](/tutorial/error-handling/#catch-statements) |
| `yeet` | Throws an error. | `yeet Error("error message")` | [Yeet](/tutorial/error-handling/#yeet) |
| `error` | Declares an error type. | `error MyError { ... }` | [Error Declaration](/tutorial/error-handling/#error-declaration) |
| `!` | Bang operator. | `let x = foo()!` | [Bang](/tutorial/error-handling/#bang-operator) |