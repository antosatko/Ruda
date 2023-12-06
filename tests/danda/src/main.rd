import "#io"
import "#fs"
import "#string"

import "danda.rd"


fun main() {
    let file = fs.File("test.txt")
    let content = file.read()
    io.println(content)
}