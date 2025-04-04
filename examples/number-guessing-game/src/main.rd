//import "std.io"
import "#io"
import "#string"


fun main() {
    let answer = 42
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
            io.println("You win! It took you " + guesses + " guesses.")
            break
        }
    }
}