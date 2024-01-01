import "#window" as win

fun handle(settings: Settings, event: win.Event) {
    if event.code() as int == win.Events.KeyPressed {
        let key = event.key() as int
        if key == win.Keys.Left {
            settings.left = 0
        }
        if key == win.Keys.Right {
            settings.right = 0
        }
        if key == win.Keys.Up {
            settings.up = 0
        }
        if key == win.Keys.Down {
            settings.down = 0
        }
    }
    if event.code() as int == win.Events.KeyReleased {
        let key = event.key() as int
        if key == win.Keys.Left {
            settings.left = -1
        }
        if key == win.Keys.Right {
            settings.right = -1
        }
        if key == win.Keys.Up {
            settings.up = -1
        }
        if key == win.Keys.Down {
            settings.down = -1
        }
    }
}



struct Settings {
    // keys
    // -1 = not pressed
    // 0 = pressed this frame
    // n+ = pressed for n frames
    left: int
    right: int
    up: int
    down: int

    new() {
        self.left = -1
        self.right = -1
        self.up = -1
        self.down = -1
    }

    fun frame(self) {
        if self.left >= 0 {
            self.left += 1
        }
        if self.right >= 0 {
            self.right += 1
        }
        if self.up >= 0 {
            self.up += 1
        }
        if self.down >= 0 {
            self.down += 1
        }
    }

    fun kDown(n: int): bool {
        return n >= 0
    }

    fun kPressed(self, n: int): bool {
        return n == 0
    }
}