import "#io"
import "#fs"
import "#string"

import "danda.rd"

struct Danda {
    a: [int]

    new() {
        self.a = [1, 2, 3]
    }
}


fun main() {
    let d = Danda()
    io.println(d.a[2]) 
}