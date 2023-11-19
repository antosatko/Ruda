import "#io"

fun greet(name: string) {
    io.println(&"Hello, " + name + &"!")
}

fun goodbye(name: string) {
    io.println(&"Goodbye, " + name + &"!")
}

fun expensive(): number {
    let i = 0
    while i < 10000000 {
        i = i + 1
    }
    return i
}