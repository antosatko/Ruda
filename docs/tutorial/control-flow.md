# Control flow

Control flow lets you control the order in which your code is executed. It allows you to add logic to your programs.

## Conditional statements

Conditional statements are used to execute certain code only if a certain condition is met.

```ruda
if true {
    io.println("This will be printed")
}

if false {
    io.println("This will not be printed")
}
```

This will print `This will be printed` to the console.

### Else

You can use the `else` keyword to execute code if the condition is not met.

```ruda
if false {
    io.println("This will not be printed")
} else {
    io.println("This will be printed")
}
```

This will print `This will be printed` to the console.

### Else if

You can use the `else if` keyword to execute code if the condition is not met and another condition is met.

```ruda
if false {
    io.println("This will not be printed")
} else if true {
    io.println("This will be printed")
} else {
    io.println("This will not be printed")
}
```

This will print `This will be printed` to the console.

After one of the conditions is met, the rest of the conditions are not checked.

# Switch statements

> ⚠️ Unstable feature ⚠️

Switch statements are used to execute different code depending on the value of a variable.

```ruda
let x = 5

switch x {
    0 {
        io.println("x is 0")
    },
    1 {
        io.println("x is 1")
    },
    5 {
        io.println("x is 5")
    },
    _ {
        io.println("x is not 0, 1 or 5")
    }
}
```

This will print `x is 5` to the console.

The `_` case is the default case. It is executed if none of the other cases are executed.

# Loops

Loops are used to execute code multiple times.

## Loop

The `loop` keyword is used to create an infinite loop.

```ruda
loop {
    io.println("This will be printed forever")
}
```

This will print `This will be printed forever` to the console forever.

## While

The `while` keyword is used to create a loop that executes while a certain condition is met.

```ruda
let x = 0

while x < 5 {
    io.println(x)
    x += 1
}
```

This will print `0`, `1`, `2`, `3` and `4` to the console.

## For

> ⚠️ Unstable feature ⚠️

The `for` keyword is used to create a loop that executes for each item in a collection.

```ruda
let numbers = [1, 2, 3, 4, 5]

for number in numbers {
    io.println(number)
}
```

This will print `1`, `2`, `3`, `4` and `5` to the console.

## Break

The `break` keyword is used to exit a loop.

```ruda
let x = 0

while true {
    io.println(x)
    x += 1

    if x == 5 {
        break
    }
}
```

This will print `0`, `1`, `2`, `3` and `4` to the console.

## Continue

The `continue` keyword is used to skip the rest of the loop and continue to the next iteration.

```ruda
let numbers = [1, 2, 3, 4, 5]

for number in numbers {
    if number == 3 {
        continue
    }

    io.println(number)
}
```

This will print `1`, `2`, `4` and `5` to the console.

## Labels

Breaking targets only the nearest loop:

```ruda
loop {
    loop { // <--- this will exit
        break
    }
}
// results in infinite loop
```

To prevent this behaviour you can label your loops and later use the label to specify which loop you want to break.

```ruda
loop "myLoop": { break "myLoop" }
while "myWhile": true { break "myWhile" }
for "myFor": i in arr { break "myFor" }
```

```ruda
loop "main": { // <--- this will exit
    loop "secondary": {
        loop {
            break "main"
        }
    }
    io.println("You will never see this Mark!")
}
```

Same can be applied to continues

```ruda
loop "a": { continue "a" } // infinite loop that ends on continue statement
```
