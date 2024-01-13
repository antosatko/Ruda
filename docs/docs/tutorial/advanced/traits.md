# Traits

> ⚠️ Will be implemented together with generics in future updates ⚠️

Traits are a way to define a set of methods that a type can implement. Traits are similar to interfaces in other languages.

Also see [Generics](/tutorial/advanced/generics), where traits are used to define constraints on type parameters.

## Syntax

Traits are declared using the `trait` keyword. The methods are declared using the `fun` keyword.

```ruda
trait Foo {
    fun foo(name: string)
    fun bar()!: int
}
```

## Implementing traits

A type can implement a trait using the `impl` keyword. The methods are implemented using the `fun` keyword.

```ruda
impl Bar trait Foo {
    fun foo(name: string) {
        io.println("Hello, " + name)
    }

    fun bar()!: int {
        anEvilFunctionThatCrashesTheProgramOnFriday()
        return 34
    }
}
```

## Trait constraints

Traits can be used as constraints on type parameters. This means that the type parameter must implement the trait.

First we define a trait:

```ruda
trait Animal {
    fun speak()
}
```

Then we define two types that implement the trait:

```ruda
struct Dog {
    name: string,
}

impl Dog trait Animal {
    fun speak() {
        io.println("Woof!")
    }
}

struct Cat {
    name: string,
}

impl Cat trait Animal {
    fun speak() {
        io.println("Meow!")
    }
}
```

> Don't forget to implement a constructor for each type.

Then we can write a pair structure that can hold any two types that implement the `Animal` trait:

```ruda
struct Pair<T(Animal), U(Animal)> {
    x: T,
    y: U,
}

impl Pair {
    fun Pair(x: T, y: U) {
        this.x = x
        this.y = y
    }
}
```

Using this structure, we can create a pair of a dog and a cat:

```ruda
fun main() {
    let pair = Pair(Dog("Fido"), Cat("Mittens"))
    pair.x.speak()
    pair.y.speak()
}

// Output:
// Woof!
// Meow!
```
