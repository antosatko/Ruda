import "#io"
import "#fs"

import "main.rd"

fun file(): string {
    return fs.fileRead("src/main.rd")
}

fun samik(a: float, b: float): float {
    return a + b
}

fun qwe(): File{
    fs.fileWrite("ahoj", "cau")
    fs.fileAppend("ahoj", "neoc"+ '\n')
    return fs.fileOpen("asd")
}





fun g(a: number): number {
    loop {
        if false{

        }
        else{{{{{{
            io.println(a)
            if a <= 1 {
                return 1
            }
            let b = main.f(a - 1)
            return b
        }}}}}}
    }
}