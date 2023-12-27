import "#io"
import "#memory"

fun factorial(n: int): int {
    if 0 == n {return 1}
    return n * factorial(n - 1)
}

fun fib(n: uint): uint {
    if n == 0 {
        return 0
    }

    if n == 1 {
        return 1
    }

    return fib(n - 1) + fib(n - 2)
}





fun main() {
    loop {
        "ahoj" + " " + "svete"
    }
}