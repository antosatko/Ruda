# Hello world

Where else to start than with the classic "Hello world" example.

Before we start, make sure you have the Ruda virtual machine installed. You can find instructions on how to do that [here](../installation.md).

When you create a new project, Ruda will automatically create a `main.rd` file in the `src` directory. This is where you will write your code.

Open the `main.rd` file. You should see the following code:

```ruda
import "#io"

fun main() {
    io.println("Hello world")
}
```

Let's go through this line by line.

```ruda
import "#io"
```

This line imports the `io` module from the standard library. This module allows you to read and write to the console.

```ruda
fun main() {
```

This line defines a function called `main`. This is the entry point of the program. The program will start executing from here.

```ruda
    io.println("Hello world")
```

This line calls the `println` function from the `io` module. This function prints the given string to the console.

```ruda
}
```

This line ends the `main` function.

## Running the program

To run the program, run the following command:

```bash
ruda run
```

This will build and run the program. If you did everything correctly, you should see the following output:

```
Hello world
```

Congratulations! You have successfully written and run your first Ruda program.
