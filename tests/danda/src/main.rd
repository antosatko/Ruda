import "#io"
import "#fs"
import "#string"

import "danda.rd"


fun main() {
    let a = [50; 3]
    a[1] = 100
    io.println(a[1])
}