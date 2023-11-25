# Memory Management

Ruda uses a garbage collector to manage memory. This means that you don't have to worry about freeing memory yourself. However, this does not mean that you can't manage memory yourself. Ruda provides a way to allocate and free memory manually.

## Allocating memory

You can allocate memory using the `new` keyword. This allocates memory on the heap and returns a pointer to the allocated memory.

```ruda
let x = new 5
```

Heap can be allocated for any type, including structs, enums, and arrays.

```ruda
struct Foo {
    x: int,
}

impl Foo {
    fun Foo(x: int) {
        self.x = x
    }
}

let foo = new Foo(5)
```

## Freeing memory

Freeing memory will be part of the standard library in the future. For now, you just need to rely on the garbage collector (not that it doesn't do a good job).