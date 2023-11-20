import "#io"

import "soubor.rd"


struct Danda {
    a: int?
    b: int?

    fun Danda (a: int, b: int) {
        self.a = a
        self.b = b
    }

    fun add(self): int {
        return self.a + self.b
    }

    fun blabla(): int {
        return 5
    }
}

fun main() {
    let a = !5
    let b = 6 + b
    b += 90
}

