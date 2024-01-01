import "#io"
import "#window" as win

import "events.rd"
import "assets.rd"

struct Player {
    x: float
    y: float
    w: float
    h: float
    speed: float
    hitbox: float
    color: win.Color

    ctx: win.Window

    new(x: float, y: float, ctx: win.Window) {
        self.x = x
        self.y = y
        self.w = 25
        self.h = 50
        self.hitbox = 5
        self.speed = 5
        self.color = win.Color(255, 255, 255, 255)
        self.ctx = ctx
    }

    fun draw(self) {
        let ctx = self.ctx
        // player rectangle
        ctx.color(self.color)
        ctx.drawRectangle(self.x - self.w / 2, self.y - self.h / 2, self.w, self.h)
        // player hitbox
        ctx.color(win.Color(255, 0, 0, 255))
        ctx.drawCircle(self.x - self.hitbox, self.y - self.hitbox, self.hitbox)
    }

    fun frame(self, settings: events.Settings) {
        let ctx = self.ctx
        if settings.kDown(settings.left) {
            self.x -= self.speed
            if self.x - self.w / 2 < 0 {
                self.x = self.w / 2
            }
        }
        if settings.kDown(settings.right) {
            self.x += self.speed
            if self.x + self.w / 2 > 900 {
                self.x = 900 - self.w / 2
            }
        }
        if settings.kDown(settings.up) {
            self.y -= self.speed
            if self.y - self.h / 2 < 0 {
                self.y = self.h / 2
            }
        }
        if settings.kDown(settings.down) {
            self.y += self.speed
            if self.y + self.h / 2 > 600 {
                self.y = 600 - self.h / 2
            }
        }
    }
}

struct PlayerShot {
    x: float
    y: float
    w: float
    h: float
    speed: float
    color: win.Color

    ctx: win.Window
    image: win.Image

    new(x: float, y: float, ctx: win.Window, assets: assets.Assets) {
        self.x = x
        self.y = y
        let image = assets.playerShot
        self.image = image
        self.w = image.width()
        self.h = image.height()
        self.speed = 10
        self.color = win.Color(255, 255, 255, 255)
        self.ctx = ctx
    }

    fun draw(self) {
        let ctx = self.ctx
        ctx.drawImage(self.x - self.w / 2, self.y - self.h / 2, self.image)
    }

    fun frame(self) {
        self.y -= self.speed
    }
}