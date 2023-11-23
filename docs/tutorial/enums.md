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

# Methods

Simmilar to structs, enums can also have methods.

We will talk about methods in detail in the [Methods](/docs/tutorial/methods.md) section.

# Variant Matching

Variant matching is used to match the variant of an enum.

```ruda
enum Color {
    Red,
    Green,
    Blue
}

fun main() {
    let myColor = Color.Red

    match myColor {
        Color.Red => {
            io.println("Red")
        }
        Color.Green => {
            io.println("Green")
        }
        Color.Blue => {
            io.println("Blue")
        }
    }
}
```

# Constructors

Enums allow you to define constructors either for the enum itself or for each variant.

## Enum Constructor

```ruda
enum Color {
    Red,
    Green,
    Blue
}

impl Color {
    fun Color() {
        io.println("Red is the default color")
        self = Color.Red
    }
}
```

## Variant Constructor

```ruda
enum Color {
    Red,
    Green,
    Blue
}

impl Color {
    fun Red() {
        io.println("i am set to red")
    }
    fun Green() {
        io.println("i am set to green")
    }
    fun Blue()! {
        // blue is not implemented yet, the function will yeet an error
    }
}
```

> Note: Use this only if you really need to. It is not recommended to use constructors for enums. I don't even know why I added this feature tbh.