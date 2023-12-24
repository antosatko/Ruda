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

    
    let style = win.DrawStyle()
        .font(win.FontUbuntuMono())
        .fontSize(20)
        .color(win.ColorFrom(win.Colors.White))
        .outlineColor(win.ColorFrom(win.Colors.Green))
        .outlineThickness(10)


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

        ctx.save()
        ctx.drawRectangle(0, 0, 100, 100)
        ctx.outlineColor(win.ColorFrom(win.Colors.Blue))
        ctx.outlineThickness(10)
        ctx.drawRectangle(200, 0, 100, 100)
        ctx.color(win.ColorFrom(win.Colors.Red))
        ctx.drawRectangle(100, 100, 100, 100)
        ctx.drawCircle(100, 0, 50)
        ctx.styledText(100, 100, "Hello World!"+'\n'+"Danda", style)
        ctx.styledRectangle(200, 200, 100, 100, style)
        ctx.restore()
        
        ctx.display()
        i += 1
        if t.elapsed() > 1f {
            io.println("closing")
            ctx.close()
            break "main_loop"
        }
    }
    io.println("elapsed: " + t.elapsed())
    io.println("frames: " + i)
}