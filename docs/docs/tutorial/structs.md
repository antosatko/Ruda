# Structs

As you may have noticed, we already saw a struct in the previous section. A struct is a collection of named values. Structs are useful for grouping related data together.

Structs are declared using the `struct` keyword.

```ruda
struct Person {
    name: string,
    age: int
}
```

This declares a struct named `Person` with two fields: `name` and `age`.

## Creating structs

Structs can be created using their constructor. The constructor is the struct name followed by parentheses.

```ruda
let person = Person("Terry", 34)
```

## Constructor

The code above won't work because we haven't defined a constructor for the `Person` struct because Ruda couldn't generate a constructor for the struct because it has a field of type `string`.

We can define a constructor for the `Person` struct like this:

```ruda
struct Person {
    name: string,
    age: int
}

impl Person {
    fun Person(name: string, age: int) {
        self.name = name
        self.age = age
    }
}
```

Don't look too much into all the keywords and syntax yet. We will cover them in later sections.

For now just know that constructor is a function that is called when a struct is created. The constructor is used to initialize the struct fields (name, age).

Self is a special variable that refers to the struct instance. We use self to access the struct fields.

Now we can create as many `Person` structs as we want.

```ruda
let person1 = Person("Terry", 34)
let person2 = Person("Danda", 19)
```

## Accessing fields

Struct fields can be accessed using the dot operator.

```ruda
let person = Person("Terry", 34)

io.println(person.name) // Terry
io.println(person.age) // 34
```

## Updating fields

Struct fields can be updated using the dot operator.

```ruda
let person = Person("Terry", 34)

person.name = "Danda"
person.age = 19

io.println(person.name) // Danda
io.println(person.age) // 19
```