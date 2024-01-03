import "#io"
import "#window" as win
import "#time"

fun handleEvent(event: win.Event, keys: Keys) {
    let code = event.code() as int
    if code == win.Events.KeyPressed {
        let key = event.key() as int
        if key == win.Keys.W {
            keys.up = 0
        }
        if key == win.Keys.S {
            keys.down = 0
        }
        if key == win.Keys.D {
            keys.right = 0
        }
        if key == win.Keys.A {
            keys.left = 0
        }
    }else if code == win.Events.KeyReleased {
        let key = event.key() as int
        if key == win.Keys.W {
            keys.up = -1
        }
        if key == win.Keys.S {
            keys.down = -1
        }
        if key == win.Keys.D {
            keys.right = -1
        }
        if key == win.Keys.A {
            keys.left = -1
        }
    }
}

struct Projectile {
    x: float
    y: float
    r: float
    xs: float
    ys: float
    color: win.Color
    ctx: win.Window

    new (rng: time.Rng, ctx: win.Window) {
        self.x = rng.range(-100, 1700)
        self.y = rng.range(-100, 1000)
        if rng.coin() {
            self.x *= -1
        }
        if rng.coin() {
            self.y *= -1
        }
        self.r = rng.range(8, 20)
        self.xs = rng.range(-8, 8)
        self.ys = rng.range(-8, 8)
        self.color = win.Color(255, 255, 255, 255)
        self.ctx = ctx
    }

    fun move(self) {
        self.x += self.xs
        self.y += self.ys
    }

    fun draw(self) {
        let ctx = self.ctx
        ctx.color(self.color)
        ctx.drawCircle(self.x - self.r, self.y - self.r, self.r)
    }
}

struct Keys {
    up: int
    down: int
    right: int
    left: int

    new() {
        self.up = -1
        self.down = -1
        self.right = -1
        self.left = -1
    }

    fun frame(self) {
        self.up = keyFrame(self.up)
        self.down = keyFrame(self.down)
        self.right = keyFrame(self.right)
        self.left = keyFrame(self.left)
    }
}

fun keyFrame(key: int): int {
    if key == -1 {
        return key
    }
    return key + 1
}

struct Player {
    x: float
    y: float
    r: float
    ctx: win.Window
    color: win.Color

    new(x:float,y:float, ctx: win.Window) {
        self.x = x
        self.y = y
        self.r = 5
        self.color = win.Color(255,60,60,255)
        self.ctx = ctx
    }

    fun draw(self) {
        let ctx = self.ctx
        ctx.color(self.color)
        ctx.drawCircle(self.x - (self.r), self.y - (self.r), self.r)
    }

    fun move(self, keys: Keys) {
        if keys.up >= 0 {
            self.y -= 12
            if self.y - self.r < 0 {
                self.y = self.r
            }
        }
        if keys.down >= 0 {
            self.y += 12
            if self.y + self.r > 900 {
                self.y = 900 - self.r
            }
        }
        if keys.left >= 0 {
            self.x -= 12
            if self.x < self.r {
                self.x = self.r
            }
        }
        if keys.right >= 0 {
            self.x += 12
            if self.x + self.r > 1600 {
                self.x = 1600 - self.r
            }
        }
    }
}

fun main() {
    let ctx = win.WinBuilder().width(1600).height(900).default().build()
    let event: win.Event?
    ctx.background(win.Color(0,0,0,255))
    ctx.fps(60)
    let keys = Keys()

    let player = Player(500, 500, ctx)

    let rng = time.Rng()
    let projectiles: [Projectile] = []
    let i = 0
    while i < 50 {
        projectiles.push(Projectile(rng, ctx))
        i += 0
    }
    loop "main": {
        keys.frame()
        loop "events": {
            event = ctx.poll()
            if !event? {
                break "events"
            }
            if event.code() as int == win.Events.Closed {
                break "main"
            }
            handleEvent(event, keys)
        }
        ctx.clear()

        player.move(keys)
        player.draw()
        

        ctx.display()
    }

    ctx.close()
}