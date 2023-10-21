# Operator Overloading

This is a feature that allows you to define the behavior of operators when applied to your own types. For example, you can define what happens when you add two instances of your class together, or when you compare them for equality.

## Syntax

Operator overloading is done inside the `impl` block of a type. The operator is defined using the `operator` keyword.

```ruda
struct Foo {
    x: int,
}

impl Foo {
    operator + (other: Foo): Foo {
        return Foo(self.x + other.x)
    }

    operator == (other: Foo): bool {
        return self.x == other.x
    }

    fun Foo(x: int) {
        self.x = x
    }
}
```

## Traits

You can also define operators for [traits](/tutorial/advanced/traits). This is useful if you want to define operators for multiple types.

```ruda
trait Add {
    operator + (other: Self): Self
}

struct Foo {
    x: int,
}

impl Foo trait Add {
    operator + (other: Foo): Foo {
        return Foo(self.x + other.x)
    }
}
```

## Overloadable operators

The following operators can be overloaded:

- `+`
- `-`
- `*`
- `/`
- `%`
- `==`
- `!=`
- `<`
- `>`
- `<=`
- `>=`
- `!`
- `&&`
- `||`
- `&`
- `|`
- `[`