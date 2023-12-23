import "#io"
import "#fs"
import "#string"
import "#time"
import "#window" as win

import "danda.rd"

enum TestEnum {
    A
    B
    C
    Danda
}

fun tenum_tostr(e: TestEnum): string {
    if e as uint == TestEnum.A {
        return "A"
    } else if e as uint == TestEnum.B {
        return "B"
    } else if e as uint == TestEnum.C {
        return "C"
    } else {
        return "Unknown"
    }
}

fun main() {
    let winBuilder = win.WinBuilder()
    winBuilder.width(160)
    winBuilder.height(90)
    let ctx = win.Window("Test", winBuilder)
    ctx.clear()

    // event poll ..

    // drawing ..

    ctx.display()

    let d = danda.Danda(60)
    io.println(d.a as float)

    ctx.close()
    
    io.println(tenum_tostr(2))
    win.Event.Closed
}