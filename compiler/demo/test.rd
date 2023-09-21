// import "std.time" as time
// import "std.window" as win

import "variables.rd"
import "test.rd"

pub const S_WIDTH = 650
const S_HEIGHT = 400


pub trait TRT impl danda.dfg5 .fg, sger.fdg {

    fun constructor(direction: float){
        self.x = -S_WIDTH as float / 2f
        self.y = S_HEIGHT as float / 2f
        self.r = 5f
        self.xs = direction
        self.ys = 0f
    }
}

struct Ball<Generic(tr)> {
    x: float,
    y: float,
    r: float,
    xs: float,
    ys: float,
}

impl Ball {
    fun constructor(direction: float){
        self.x = S_WIDTH as float / 2f
        self.y = S_HEIGHT as float / 2f
        self.r = 5f
        self.xs = direction
        self.ys = 0f
    }
    fun move(&self) {
        self.x += self.xs
        self.y += self.ys
    }
    fun draw(&self, ctx: &win.Window) {
        ctx.fillColor(win.Colors.White)
        ctx.fillCircle(self.x, self.y, self.r)
    }
}

enum Sides {
    Left = 50i,
    Right = 600i,
}

struct Player {
    side: Sides,
    y: float,
    w: float,
    h: float,
    speed: float,
    points: uint
}

impl Player {
    fun constructor(side: Sides){
        self.y = 0f
        self.w = 20f
        self.h = 100f
        self.speed = 1f
        self.points = 0
        self.side = side
    }
    fun move(&self, direction: float) {
        self.y += self.speed * direction
        if self.y < 0 {
            self.y = 0
        }
        else if self.y > S_HEIGHT - self.h {
            self.y = S_HEIGHT - self.h
        }
    }
    fun collision(&self, ball: &Ball) {
        if self.side as int < ball.x + ball.r / 2 && self.side as int + self.w > ball.x - ball.r / 2 &&
            self.y < ball.y + ball.r / 2 && self.y + self.h > ball.y - ball.r / 2 
        {
            // collision detected
            // too lazy to do something rn
            ball.xs *= -1f
        }
    }
    fun draw(&self, ctx: &win.Window) {
        ctx.fillColor(win.Colors.White)
        ctx.fillRect(self.Sides as float, self.y, self.w, self.h)
    }
}

fun draw(p1: &Player, p2: &Player, ball: &Ball, ctx: &win.Window){
    ball.draw()
    p0.draw()
    p1.draw()
    // kdybych nebyl liny tak bych ted vykreslil skore atd..
}

fun main(){
    let ctx = win.init()
    ctx.title("myGame")
    let players = [Player(Sides.Left), Player(Sides.Right)]
    let ball = Ball(1f)
    let running = true
    let gameRunning = true
    while running {
        for e in ctx.get_events() {
            switch e.kind {
            win.EventType.Close {
                running = false
            },
            win.EventType.KeyDown {
                if e.key == win.Keys.S {
                    players[0].move(1f)
                }
                else if e.key == win.Keys.W {
                    players[0].move(-1f)
                }
                else if e.key == win.Keys.ArrowDown {
                    players[1].move(1f)
                }
                else if e.key == win.Keys.ArrowUp {
                    players[1].move(-1f)
                }
            }
            }
        }
        // Game logic
        if gameRunning {
            draw(&players[0], &players[1], &ball, &ctx)
        }
        if players[0].points == 10 || players[1].points == 10 {
            gameRunning = false
        }
    }
}
