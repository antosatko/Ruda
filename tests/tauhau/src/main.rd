import "#io"
import "#window" as win
import "#memory"
import "#time"

import "events.rd"
import "units.rd"
import "assets.rd"


fun main() {
    let ctx = win.WinBuilder()
        .title("Tauhau")
        .width(900)
        .height(600)
        .default()
        .build()
    ctx.fps(60)


    let assets = assets.Assets()
    let player = units.Player(450, 500, ctx)
    let settings = events.Settings()
    let playerShots: [units.PlayerShot] =  []

    playerShots.push(units.PlayerShot(450, 500, ctx, assets))

    let t = time.Clock()
    let frame = 0
    let i = 0

    let event: win.Event?
    ctx.background(win.Color(0, 0, 0, 255))
    loop "main": {
        loop "event": {
            event = ctx.poll()
            if event {
                break "event"
            }
            if event.code() as int == win.Events.Closed {
                break "main"
            }
            events.handle(settings, event)
        }
        ctx.clear()

        settings.frame()
        player.frame(settings)

        i = 0
        while i < playerShots.len() {
            let shot = playerShots[i]
            shot.frame()
            shot.draw()
            i += 1
        }

        player.draw()

        memory.Gc.sweep()
        ctx.display()

        frame += 1
        // this is for testing purposes
        // it will break the main loop after 1 second
        // so that I can see how many frames were rendered
        /*if t.elapsed() > 1 {
            break "main"
        }*/
    }
    io.println("frames: " + frame)
}