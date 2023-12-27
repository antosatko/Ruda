import "#io"
import "#fs"
import "#string"
import "#time"
import "#window" as win


    fun nullable(a: int?) {
        io.println(a)
    }

fun main() {
    let ctx = win.WinBuilder()
        .width(800)
        .height(450)
        .default()
        .title("Test")
        .build()
    let event: win.Event?
    let i = 0


    nullable(null)


    io.println(win.alert("Title", "555"))
    io.println(win.prompt("Title", "Message", "Default"))
    io.println(win.confirm("Title", "Message")) 
    



    
    let style = win.DrawStyle()
        .font(win.Font.ubuntuMono())
        .fontSize(20)
        .color(win.Color.From(win.Colors.Red))
        .outlineColor(win.Color.From(win.Colors.Green))
        .outlineThickness(10)
        .font(win.Font.ubuntuMono())

    let t = time.Clock()

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
        ctx.outlineColor(win.Color.From(win.Colors.Blue))
        ctx.outlineThickness(10)
        ctx.drawRectangle(200, 0, 100, 100)
        ctx.color(win.Color.From(win.Colors.Red))
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