import "#io"
import "#fs"
import "#string"
import "#time"
import "#window" as win



fun main() {
    let winBuilder = win.WinBuilder()
    winBuilder.width(160)
    winBuilder.height(90)
    let ctx = win.Window("Test", winBuilder)
    loop {}
}