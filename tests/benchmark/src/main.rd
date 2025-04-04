import "#io"
import "#memory"

fun factorial(n: int): int {
    if 0 == n {return 1}
    return n * factorial(n - 1)
}

fun fib(n: uint): uint {
    if n < 2 {
        return n
    }

    return fib(n - 1) + fib(n - 2)
}





fun main() {
    let n = 33
    fib(n)
}