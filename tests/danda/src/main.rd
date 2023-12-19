import "#io"
import "#fs"
import "#string"
import "#time"

import "danda.rd" as d


fun fib(n: uint): uint {
    if n == 0 {
        return 0
    }

    if n == 1 {
        return 1
    }

    return fib(n - 1) + fib(n - 2)
}

fun fib2(n: uint): uint {
    if n == 0 {
        return 0
    } else if n == 1 {
        return 1
    }

    let fibSeq = [0; n]
    fibSeq[0] = 0
    fibSeq[1] = 1

    let i = 2
    while i < n {
        fibSeq[i] = fibSeq[i - 1] + fibSeq[i - 2]
        i = i + 1
    }

    return fibSeq[n - 1]
}





fun main() {
    let clock = time.Clock()
    fib(33)
    io.println(clock.elapsed())
}