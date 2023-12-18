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
    io.println(b(a()))
}