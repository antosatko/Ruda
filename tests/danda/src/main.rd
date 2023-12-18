import "#io"
import "#fs"
import "#string"

import "danda.rd" as d


fun a(): int<aaa> {
    return 0
}

fun b(arg: int<aaa>): d.Danda<cccc> {
    return d.Danda(6)
}


fun main() {
    let i = 0
    while "ahoj": true {
        i += 1;
        if i == 5 {
            continue "ahoj"
        }
        if i == 10 {
            break "ahoj"
        }
        io.println(i)
    }
}