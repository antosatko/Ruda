import "#window" as win

struct Assets {
    playerShot: win.Image

    new() {
        self.playerShot = win.Image("assets/playerShot.png")
    }
}