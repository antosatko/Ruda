import "#io"
import "#fs"
import "#string"

import "danda.rd"


fun a<T>(arg: T): T? {
    return null
}


fun main() {
    let b = a(1)
    io.println(b)

    let arr = [1, 6, 9]
    arr.push(3)

    io.println(arr.len())
}