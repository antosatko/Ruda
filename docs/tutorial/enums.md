# Enums

Enums in Ruda are quite boring. They are used for defining a type that can have a finite number of numbers.

```ruda
enum Color {
    Red,
    Green,
    Blue
}
```

You can also assign numbers to enum variants.

```ruda
enum Color {
    Red, // 0
    Green = 5, // 5
    Blue // 6
}
```

# Variant Matching

There is no "ergonomic" way of matchng enums in the current version. To match two enums you need to cast one to any number and compare them.

```ruda
enum Color {
    Red,
    Green,
    Blue
}

fun main() {
    let myColor = Color.Red

    if myColor as int == Color.Red {
        io.println("The value is indeed red")
    } else {
        io.println("The value does not seem to be red")
    }
}
```
