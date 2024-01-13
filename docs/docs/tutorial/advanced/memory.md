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

## Working with Garbage collector

Many languages implement GC to ensure memory safety and ease the development cycle. While all this is also true for Ruda, it takes a slightly different approach. Garbage collector manages memory, but the user manages the garbage collector. It is as easy as calling a method through the memory API in standard library.

```ruda
import "#memory"

fun main(){
    memory.Gc.sweep()
}
```

> You right now: "_WHY? This has to be a step bacwards.._"

You may be right. The difference between good and bad GC is that you do not even notice the good one, therefore my implementation would be the worst possible. In the real world things are not so simple for anyone to decide whats the best memory model. Each has their uses. My offers ease of use combined with controll. In garbage collected languages you often dont know when the collector starts sweeping. This results in uncontrollable and unwanted lag spikes. Ruda offers more direct memory access while also hiding it when necessary.
