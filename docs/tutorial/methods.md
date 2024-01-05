# Methods

Methods are functions that are defined on a type. Thats all there is to it.

## Defining methods

Methods are defined **after** the struct fields. If the struct contains constructor, it must be the first method defined.

```ruda
struct Order {
    // fields are always first
    field1: int
    field2: float
    // ..

    // constructor follows
    new (n: int) {
        self.field1 = n
        self.field2 = 60f
    }

    // the rest of the methods
    fun set1(self, n: int){
        self.field1 = n
    }
    // ..
}
```

```ruda
struct Person {
    name: string
    age: int

    new (name: string, age: int) {
        self.name = name
        self.age = age
    }

    fun greet(self) {
        io.println("Hello, my name is " + self.name)
    }
}
```

Here we first define a struct named `Person` with two fields `name` and `age`.

Then we define a constructor for the `Person` struct, this is the same constructor we defined in the [Structs](/tutorial/structs) section.

Then we define a method named `greet` that prints a greeting message.

The method takes a `self` parameter. This is a special parameter that refers to the struct instance. We use `self` to access the struct fields.

## Calling methods

Methods are called using the dot operator.

```ruda
let person = Person("Terry", 34)

person.greet() // Hello, my name is Terry
```

## Self

Self is a special variable that refers to the struct instance. We use self to access the struct fields.

## Static methods

Static methods are methods that are defined on the type itself. They are called using the type name.

```ruda
struct Person {
    name: string
    age: int


    new (name: string, age: int) {
        self.name = name
        self.age = age
    }

    fun greet(self) {
        io.println("Hello, my name is " + self.name)
    }

    fun create(name: string, age: int): Person {
        return Person(name, age)
    }
}

let person = Person.create("Terry", 34)
```

Here we define a static method named `create` that creates a new `Person` struct.

## Static vs Instance methods

Static methods are called using the type name while instance methods are called using the struct instance.

Static methods are useful for creating alternative constructors.
