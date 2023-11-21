import "#io"
import "#string"

/*
import "soubor.rd"


struct Danda {
    a: int?
    b: int?

    fun Danda (a: int, b: int) {
        self.a = a
        self.b = b
    }

    fun add(self): int {
        return self.a + self.b
    }

    fun blabla(): int {
        return 5
    }
}
*/

fun main() {
    // calculator

    loop {
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

