# Number guessing game

This is probably your first time writing a program in Ruda. Good luck!

## What are we building?

In this project you will create a number guessing game. The computer will think of a predefined number between 1 and 100 and you will try to guess it. The computer will tell you if your guess is too high or too low.

## Requirements

- The user should be able to guess the number.
- The computer should tell the user if their guess is too high or too low.
- The user should be able to quit the game at any time.
- The user should be told how many guesses it took them to guess the number when they win.

## Example

```bash
Welcome to the number guessing game!
I am thinking of a number between 1 and 100.
Guess the number: 50
Too high!
Guess the number: 25
Too low!
Guess the number: 37
Too high!
Guess the number: 31
Too high!
Guess the number: 28
Too low!
Guess the number: 29
You win! It took you 6 guesses.
Do you want to play again? (y/n): n
```

## Tips

- Use the `io.inputln()` function to read user input.
- Use the `io.println()` function to print output.
- String library contains `parse()` function that can be used to convert user input to a number.
- The generated number can be constant.

## Strategy

> This is a good time to start thinking about how you will structure your code.

In my head I'm focusing on the following parts:
- Ability to ask the user for input until they guess the correct number.
- Changing the game answer based on the user's input.
- Keeping track of how many guesses the user has made.
- Telling the user how many guesses they made when they win.

## Solution

> NO CHEATING!
> Just kidding, but seriously try to solve the problem yourself before looking at the solution.

```ruda
import "#io"
import "#string"


fun main() {
    let answer = 29
    let guesses = 0

    io.println("Welcome to the number guessing game!")
    io.println("I am thinking of a number between 1 and 100.")

    loop {
        io.print("Guess the number: ")
        let guess = string.parse(io.inputln())

        guesses += 1

        if guess < answer {
            io.println("Too low!")
        } else if guess > answer {
            io.println("Too high!")
        } else {
            io.println("You win! It took you " + guesses as string + " guesses.")
            break
        }
    }
}
```

## Explanation

Let's go over the solution.

First we import the `io` and `string` libraries.

```ruda
import "#io"
import "#string"
```

Then we define the `main()` function. This is the entry point of our program.

```ruda
fun main() {
    // ...
}
```

Then we create a variable called `guesses` and assign it the value of `0`. This variable will be used to keep track of how many guesses the user has made. We also create a variable called `answer` and assign it the value of `29`. This variable will be used to store the answer to the game.

```ruda

```ruda
let guesses = 0
let answer = 29
```

After the initial setup, we print a welcome message and tell the user that we are thinking of a number between 1 and 100.

```ruda
io.println("Welcome to the number guessing game!")
io.println("I am thinking of a number between 1 and 100.")
```

Then we start a loop that will keep asking the user for input until they guess the correct number.

```ruda
loop {
    // ...
}
```

Inside the loop we ask the user to guess the number and read their input. We convert the user's input to a number using the `string.parse()` function.

```ruda
io.print("Guess the number: ")
let guess = string.parse(io.inputln())
```

Then we increment the `guesses` variable by `1` to keep track of how many guesses the user has made.

```ruda
guesses += 1
```

Then we check if the user's guess is less than the answer. If it is, we tell the user that their guess is too low.

```ruda
if guess < answer {
    io.println("Too low!")
}
```

If the user's guess is not less than the answer, we check if it is greater than the answer. If it is, we tell the user that their guess is too high.

```ruda
} else if guess > answer {
    io.println("Too high!")
}
```

If the user's guess is not less than or greater than the answer, it must be equal to the answer. In this case we tell the user that they won and how many guesses it took them to guess the number.

```ruda
} else {
    io.println("You win! It took you " + guesses as string + " guesses.")
    break
}
```

Finally we break out of the loop and the program ends.