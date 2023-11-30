import "#io"
import "#fs"
import "#string"

import "danda.rd"



fun main() {
    let a = danda.Danda(6)
    let b: int = 6
    let c = a.a
    a.a = 7
    io.println(a.a)
}


// highlight self