# Generics

> ⚠️ Experimental feature ⚠️: While simple examples work this is still in early development and more useful use cases are not supported yet. It is **not recommended** to use this.

Generics are a feature that allows you to write code that can be reused for different types. For example, you might want to write a function that returns the first element of any array. This is a generic function because it can be used for any type of array, like an array of strings or an array of numbers.

```ruda
fun first<T>(array: [T]): T {
    return array[0]
}

fun main() {
    let array = ["hello", "world"]
    let first = first(array)
}
```

In this example, the `first` function is generic because it can be used for any type of array. The `T` in the function signature is a type parameter. It can be replaced with any type when the function is called.

## Syntax

Generics are enclosed in angle brackets (`<` and `>`). The type parameters are separated by commas (`,`) and each type parameter can have a [constraint](#constraints).

```ruda
fun foo<T, U>(x: T, y: U): T {
    return x
}

struct Bar<T, U> {
    x: T,
    y: U,
}

trait Baz<T, U> {
    fun foo(x: T, y: U): T
}
```

## Constraints

Type parameters can have constraints. A constraint is a type that the type parameter must be a subtype of. For example, if we want to write a function that takes two numbers and returns the larger one, we can use the `Comparable` trait as a constraint.

```ruda
fun max<T(Comparable)>(x: T, y: T): T {
    if x > y {
        return x
    } else {
        return y
    }
}
```

In this example, the `T` type parameter must be a subtype of the `Comparable` trait. This means that the `T` type parameter must implement the `Comparable` trait.

> Note: The `Comparable` trait allows you to compare two values using the `<`, `>`, `<=`, and `>=` operators. Those types must be the same.
> This means that you can't compare `T (Comparable)` with `U (Comparable)`. `T` and `U` are not guaranteed to be the same type.
