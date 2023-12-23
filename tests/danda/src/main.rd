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
    winBuilder.width(800)
    winBuilder.height(450)
    let ctx = win.Window("Test", winBuilder)
    let event: win.Event?
    let i = 0

    let time = time.Clock()

    loop "main_loop": {
        ctx.clear()
        loop "event_loop": {
            event = ctx.poll()
            if event {
                break "event_loop"
            }




            if event.code() as int == win.Events.Closed {
                break "main_loop"
            }
            if event.code() as int == win.Events.Input {
                io.println("key pressed: " + event.input())
            }

        }
        ctx.title("Frame - " + i)

        ctx.display()
        i += 1
        if time.elapsed() > 1000000 {
            ctx.close()
            break "main_loop"
        }
    }

    io.println("elapsed: " + time.elapsed())
    io.println("frames: " + i)


    let d = danda.Danda(60)
    io.println(d.a as float)

    
    io.println(tenum_tostr(2))
    win.Events.Closed
}