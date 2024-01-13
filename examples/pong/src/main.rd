import "#io"
import "#window" as win
import "#memory"

enum Side {
    Left
    Right
}

/// Object that holds the state of the keys
struct Keys {
    P1Up: bool
    P1Down: bool
    P2Up: bool
    P2Down: bool

    new() {
        self.P1Up = false
        self.P1Down = false
        self.P2Up = false
        self.P2Down = false
    }
}

/// Paddle object
struct Paddle {
    x: float
    y: float
    width: float
    height: float
    color: win.Color
    side: Side
    score: int

    new(side: Side) {
        self.y = 0
        self.width = 10
        self.height = 60
        self.side = side
        self.score = 0

        if side as int == Side.Left {
            self.x = 100
            self.color = win.Color(255, 0, 0, 255)
        } else {
            self.x = 700 - self.width
            self.color = win.Color(0, 0, 255, 255)
        }
    }

    fun draw(self, ctx: win.Window) {
        // FIXME: self.ctx.cokoliv nefunguje
        ctx.color(self.color)
        ctx.drawRectangle(self.x, self.y, self.width, self.height)
    }

    fun move(self, y: float) {
        self.y = y
    }
}

/// Ball object
///
/// The ball is a circle that moves around the screen and bounces off the paddles
struct Ball {
    x: float
    y: float
    radius: float
    xspeed: float
    yspeed: float

    new() {
        self.x = 200
        self.y = 300
        self.radius = 10
        self.xspeed = 8
        self.yspeed = -5
    }

    /// Draw the ball
    fun draw(self, ctx: win.Window) {
        let b = (self.x / 800) * 255
        let r = 255 - b
        let g = 0
        ctx.color(win.Color(r, g, b, 255))
        ctx.drawCircle(self.x, self.y, self.radius)
    }

    /// Move the ball
    fun move(self) {
        self.x += self.xspeed
        self.y += self.yspeed

        if self.y > 600 - self.radius*2 {
            self.yspeed *= -1
            self.y = 600 - self.radius*2
        }
        if self.y < 0 {
            self.yspeed *= -1
            self.y = 0
        }
    }

    /// Check for collision with paddle
    fun collision(self, paddle: Paddle) {
        // paddle left side
        if self.xspeed > 0 && self.x > paddle.x - self.radius*2 && self.x < paddle.x + paddle.width {
            if self.y + self.radius > paddle.y && self.y - self.radius < paddle.y + paddle.height {
                self.xspeed *= -1
                self.x = paddle.x - self.radius*2

                // change yspeed based on how far from the center of the paddle the ball hit and if it was moving up or down
                let paddleCenter = paddle.y + paddle.height / 2
                let ballCenter = self.y + self.radius
                let distanceFromCenter = ballCenter - paddleCenter
                let normalizedDistance = distanceFromCenter / (paddle.height / 2)
                self.yspeed = normalizedDistance * 8
            }
        }
        // paddle right side
        if self.xspeed < 0 && self.x < paddle.x + paddle.width && self.x > paddle.x - self.radius*2 {
            if self.y + self.radius > paddle.y && self.y - self.radius < paddle.y + paddle.height {
                self.xspeed *= -1
                self.x = paddle.x + paddle.width

                // change yspeed based on how far from the center of the paddle the ball hit and if it was moving up or down
                let paddleCenter = paddle.y + paddle.height / 2
                let ballCenter = self.y + self.radius
                let distanceFromCenter = ballCenter - paddleCenter
                let normalizedDistance = distanceFromCenter / (paddle.height / 2)
                self.yspeed = normalizedDistance * 8
            }
        }
    }

    /// Check for scoring
    fun scoring(self, left: Paddle, right: Paddle) {
        if self.x < -self.radius*2 {
            right.score += 1
            self.x = 200
            self.y = 300
            self.xspeed *= -1
        }
        if self.x > 800 {
            left.score += 1
            self.x = 600 - self.radius*2
            self.y = 300
            self.xspeed *= -1
        }
    }
}

/// Handle controls
fun controlsHandler(keys: Keys, left: Paddle, right: Paddle) {
    if keys.P1Up {
        left.y -= 15
        if left.y < 0 {
            left.y = 0
        }
    }
    if keys.P1Down {
        left.y += 15
        if left.y > 600 - left.height {
            left.y = 600 - left.height
        }
    }
    if keys.P2Up {
        right.y -= 15
        if right.y < 0 {
            right.y = 0
        }
    }
    if keys.P2Down {
        right.y += 15
        if right.y > 600 - right.height {
            right.y = 600 - right.height
        }
    }
}

/// Draw UI
fun drawUi(ctx: win.Window, left: Paddle, right: Paddle) {
    ctx.fontSize(50)
    ctx.color(win.Color(255, 0, 0, 255))
    ctx.drawText(300, 10, left.score)
    ctx.color(win.Color(0, 0, 255, 255))
    ctx.drawText(500, 10, right.score)

    ctx.color(win.Color(255, 255, 255, 255))
    let i = 0
    while i < 600 {
        ctx.drawRectangle(400, i, 2, 10)
        i += 20
    }
}

/// Main function
/// Create window, paddles, ball and keys
fun main() {
    let ctx = win.WinBuilder().title("Pong").default().width(800).height(600).build()
    let left = Paddle(Side.Left)
    let right = Paddle(Side.Right)
    let ball = Ball()
    let keys = Keys()

    ctx.background(win.Color(0, 0, 0, 255))
    ctx.fps(60)

    let gameover = false

    let event: win.Event?
    loop "game": {
        loop "event": {
            event = ctx.poll()
            if !event? {
                break "event"
            }
            if event.code() as int == win.Events.Closed {
                break "game"
            }
            if event.code() as int == win.Events.KeyPressed {
                if event.key() as int == win.Keys.W {
                    keys.P1Up = true
                }
                if event.key() as int == win.Keys.S {
                    keys.P1Down = true
                }
                if event.key() as int == win.Keys.Up {
                    keys.P2Up = true
                }
                if event.key() as int == win.Keys.Down {
                    keys.P2Down = true
                }
                if event.key() as int == win.Keys.Space {
                    if gameover {
                        left.score = 0
                        right.score = 0
                        ball.x = 400 - ball.radius
                        ball.y = 300
                        ball.yspeed = 0
                        ball.xspeed = 8
                        gameover = false
                    }
                }
            }
            if event.code() as int == win.Events.KeyReleased {
                if event.key() as int == win.Keys.W {
                    keys.P1Up = false
                }
                if event.key() as int == win.Keys.S {
                    keys.P1Down = false
                }
                if event.key() as int == win.Keys.Up {
                    keys.P2Up = false
                }
                if event.key() as int == win.Keys.Down {
                    keys.P2Down = false
                }
            }
        }
        controlsHandler(keys, left, right)
        if !gameover {
            ball.move()
            ball.collision(left)
            ball.collision(right)
            ball.scoring(left, right)
        }
        ctx.clear()


        left.draw(ctx)
        right.draw(ctx)
        ball.draw(ctx)
        drawUi(ctx, left, right)

        if left.score == 10 || right.score == 10 {
            gameover = true
            ctx.fontSize(80)
            ctx.color(win.Color(0, 0, 0, 200))
            ctx.drawRectangle(0, 0, 800, 600)
            if left.score == 10 {
                ctx.color(win.Color(255, 0, 0, 255))
                ctx.drawText(200, 200, "Player 1 wins!")
            } else {
                ctx.color(win.Color(0, 0, 255, 255))
                ctx.drawText(200, 200, "Player 2 wins!")
            }
            ctx.fontSize(30)
            ctx.color(win.Color(255, 255, 255, 255))
            ctx.drawText(200, 300, "Press SPACE to play again...")
        }

        memory.Gc.sweep()
        ctx.display()
    }
    io.println("Goodbye!")
}