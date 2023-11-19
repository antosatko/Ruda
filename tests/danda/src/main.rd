import "#io"

import "soubor.rd"


fun fac(n: usize): usize {
    if n <= 1u {
        return 1u
    }
    
    return n * fac(n - 1u)
}

fun fib(n: usize): usize {
    if n == 0u {
        return 0u
    }

    if n == 1u {
        return 1u
    }

    return fib(n - 1u) + fib(n - 2u)
}


fun f(a: number): number {
    while true{
        io.println(a)
        if a <= 1 {
            io.println("aassa")
            return 1
        }
        let b = soubor.g(a - 1)
        return b
    }
}

fun main() {
    io.println(1 <= 1)

    let a = f(5)

    io.println(a)
    
}