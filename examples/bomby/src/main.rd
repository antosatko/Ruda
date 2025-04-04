import "#io"
import "#window"
import "#math"
import "#time"
import "#memory"

struct Player {
    x: float

    new(x: float) {
        self.x = x
    }

    fun update(self, keys: Keys) {
        if keys.A {
            self.x -= 2
        }
        if keys.D {
            self.x += 2
        }
    }
}

struct Enemy {
    x: float
    y: float
    dirX: float
    dirY: float

    new(x: float, y: float, dirX: float, dirY: float) {
        self.x = x
        self.y = y
        self.dirX = dirX
        self.dirY = dirY
    }

    fun update(self) {
        self.x += self.dirX
        self.y += self.dirY
    }

    fun collision(self, ball: Ball, rng: time.Rng): bool {
        if self.x < ball.x + 10 && self.x + 50 > ball.x && self.y < ball.y + 10 && self.y + 50 > ball.y {
            self.x = rng.range(200, 600)
            self.y = 0
            ball.x = 0
            ball.y = -200
            ball.timer = 600
            let algerba2 = algebra2(self.x, self.y, rng.gen() * 800, 600)
            self.dirX = algerba2[0]
            self.dirY = algerba2[1]
            return true
        }
        return false
    }

    fun isGameOver(self): bool {
        if self.y > 650 {
            window.alert("Game Over", "You lost")
        }
        return self.y > 650
    }
}

struct Ball {
    x: float
    y: float
    dirX: float
    dirY: float
    timer: int

    new(x: float, y: float, dirX: float, dirY: float) {
        self.x = x
        self.y = y
        self.dirX = dirX
        self.dirY = dirY
        self.timer = 0
    }

    fun update(self) {
        self.x += self.dirX;
        self.y += self.dirY;
        self.timer += 1
    }
}

struct Keys {
    A: bool
    D: bool

    new() {
        self.A = false
        self.D = false
    }
}

fun algebra2(p1x: float, p1y: float, p2x: float, p2y: float): [float] {
    let x = p2x - p1x
    let y = p2y - p1y
    let bigNumba = x * x + y * y
    let c = math.sqrt(bigNumba)
    let x1 = x / c
    let y1 = y / c
    return [x1, y1]
}

fun main() {
    let rng = time.Rng()
    let win = window.WinBuilder().title("Bomby").width(800).height(700).titlebar().close().build()

    win.fps(60)
    let event: window.Event?

    let player = Player(0)
    let ball = Ball(0, -200, -1, -1)
    ball.timer = 600
    let keys = Keys()

    let enemy1 = Enemy(0, 0, 1, 1)
    let enemy2 = Enemy(0, -150, 1, 1)
    let enemy3 = Enemy(0, -300, 1, 1)
    let score = 0

    let scoreStyle = window.DrawStyle()
    scoreStyle.font(window.Font.roboto())
    scoreStyle.color(window.Color(255, 0, 0, 255))


    loop "game": {
        loop "event": {
            event = win.poll()
            if !event? {
                break "event" 
            }
            if event.code() as int == window.Events.Closed {
                break "game"
            }
            if event.code() as int == window.Events.KeyPressed {
                if event.key() as int == window.Keys.A {
                    keys.A = true
                }
                if event.key() as int == window.Keys.D {
                    keys.D = true
                }
            }
            if event.code() as int == window.Events.KeyReleased {
                if event.key() as int == window.Keys.A {
                    keys.A = false
                }
                if event.key() as int == window.Keys.D {
                    keys.D = false
                }
            }
            if event.code() as int == window.Events.MousePressed && ball.timer > 150 {
                let p1x = player.x + 25
                let p1y = 500
                let p2x = event.mouseX() as float
                let p2y = event.mouseY() as float
                let algerba2 = algebra2(p1x, p1y, p2x, p2y)
                ball = Ball(player.x + 25, 500, algerba2[0] * 5, algerba2[1] * 5)
            }
        }
        win.clear()

        player.update(keys)
        ball.update()
        enemy1.update()
        enemy2.update()
        enemy3.update()
        if enemy1.collision(ball, rng) {
            score += 1
        }
        if enemy2.collision(ball, rng) {
            score += 1
        }
        if enemy3.collision(ball, rng) {
            score += 1
        }
        if enemy1.isGameOver() {
            break "game"
        }
        if enemy2.isGameOver() {
            break "game"
        }
        if enemy3.isGameOver() {
            break "game"
        }
        win.drawRectangle(player.x, 500, 50, 50)
        win.drawCircle(ball.x, ball.y, 10)
        win.drawRectangle(enemy1.x, enemy1.y, 50, 50)
        win.drawRectangle(enemy2.x, enemy2.y, 50, 50)
        win.drawRectangle(enemy3.x, enemy3.y, 50, 50)
        win.styledText(10, 10, "Score: " + score, scoreStyle)

        memory.Gc.sweep()
        win.display()
    }
}
