import "#io"
import "#string"



fun main(): int {
    if true {
        return 9
    }else {
        return 10
    }
    // calculator

    io.println("Welcome to the calculator!")

    while true {
        io.print("Input first number: ")
        let a = string.parse(io.inputln())
        
        io.print("Input second number: ")
        let b = string.parse(io.inputln())

        io.print("Input operator: ")
        let op = io.inputln()

        io.println("Your input: " + a + " " + op + " " + b + ".")

        if op == "+" {
            io.print("Result: ")
            io.println(a + b)
        } else if op == "-" {
            io.print("Result: ")
            io.println(a - b)
        } else if op == "*" {
            io.print("Result: ")
            io.println(a * b)
        } else if op == "/" {
            io.print("Result: ")
            io.println(a / b)
        } else {
            io.println("Unknown operator: " + op + ".")
        }

        io.println("Do you wish to continue? (y)")
        let opakuj = io.inputln() == "y"

        if !opakuj {
            return
        }
    }
}

