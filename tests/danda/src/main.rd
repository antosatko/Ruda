import "#io"
import "#fs"
import "#string"
import "#time"
import "#window" as win

fun main() {
    let winBuilder = win.WinBuilder()
    winBuilder.width(800)
    winBuilder.height(450)
    let ctx = win.Window("Test", winBuilder)
    let event: win.Event?
    let i = 0

    let t = time.Clock()

    ctx.fps(60)

    loop "main_loop": {
        ctx.clear()
        let frameStart = time.Clock()
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
        if t.elapsed() > 1 {
            io.println("closing")
            ctx.close()
            break "main_loop"
        }
    }

    io.println("elapsed: " + t.elapsed())
    io.println("frames: " + i)
}