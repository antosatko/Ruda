import "#io"
import "#fs"

import "danda.rd"

fun main(): number {
    let i = 0u
    while i < 100u {
        i = i+1u
        io.println(i) 
    }
    let name: string?
    {
        fs.fileWrite(&"test.txt", &"wysdgfed")
        let s = fs.fileRead(&"test.txt")
        io.println(s)
        io.println(&"What is your name?")
        name = io.inputln()
        danda.greet(name)
    }
    let exp = danda.expensive()
    if exp < 10000 {
        io.println(&"You are poor.")
    } else {
        io.println(&"You are rich.")
    }
    danda.goodbye(name)
    return 0
}  