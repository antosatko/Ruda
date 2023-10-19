# Pointers

We have already seen pointers in the [Data types](/tutorial/data-types/#pointers) section. In this section we will learn more about pointers.

## What is a pointer?

A pointer is a variable that stores the address of another variable. In other words, a pointer points to another variable.

## Creating a pointer

Pointers are created using the `&` operator.

```ruda
let x: int = 1

let ptr = &x
```

This creates a pointer named `ptr` that points to the variable `x`.

When we print the value of `ptr`, we get the address of `x`.

```ruda
io.println(ptr) // a big number
```

## Dereferencing a pointer

Dereferencing a pointer means accessing the value stored at the address pointed to by the pointer.

```ruda
let x: int = 1

let ptr = &x
io.println(*ptr) // 1
```

The `*` operator is used to dereference a pointer.

## Pointers to pointers

Pointers can also point to other pointers.

```ruda
let x: int = 1

let ptr1 = &x
let ptr2 = &ptr1

io.println(*ptr2) // a big number
io.println(**ptr2) // 1
```

## Why use pointers?

Pointers are useful for passing variables by reference. This means that the function can modify the variable passed to it.

```ruda
fun addOne(x: &int) {
    *x = *x + 1
}

let x: int = 1
addOne(&x)
io.println(x) // 2
```