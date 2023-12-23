import "#io"
import "#fs"
import "#string"
import "#time"
import "#window" as win

fun main() {
    let ctx = win.WinBuilder()
        .width(800)
        .height(450)
        .default()
        .title("Test")
        .build()
    let event: win.Event?
    let i = 0

    let t = time.Clock()

    ctx.fps(60)
    
    let fontStyle = win.DrawStyle()
    fontStyle.font(win.FontUbuntuMono())
    fontStyle.fontSize(20)
    fontStyle.color(win.ColorFrom(win.Colors.White))
    fontStyle.outlineColor(win.ColorFrom(win.Colors.Green))
    fontStyle.outlineThickness(1)

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


        // now with the power of rectangles at our disposal, we can draw a battle pp
        ctx.drawRectangle(0, 0, 100, 100, win.ColorFrom(win.Colors.Red))
        ctx.drawRectangle(200, 0, 100, 100, win.ColorFrom(win.Colors.Red))
        ctx.drawRectangle(100, 100, 100, 500, win.ColorFrom(win.Colors.Red))
        ctx.drawCircle(100, 0, 50, win.ColorFrom(win.Colors.Blue))
        ctx.drawText(100, 100, "Hello World!"+'\n'+"Danda", fontStyle)

        ctx.styledRectangle(200, 200, 100, 100, fontStyle)
        




        ctx.display()
        i += 1
        if t.elapsed() > 100000 {
            io.println("closing")
            ctx.close()
            break "main_loop"
        }
    }

    io.println("elapsed: " + t.elapsed())
    io.println("frames: " + i)
}