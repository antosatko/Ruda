import "#io"
import "#fs"
import "#string"

import "danda.rd"


fun a<T>(arg: T): T {
    return arg
}


fun main() {
    let b = a(1)
    io.println(b)
}