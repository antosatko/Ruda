import "#io"

struct Danda {
    a: float

    new (a: float) {
        self.a = a
    }

    fun add(self, b: float): float {
        io.println(self.a)
        return self.a + b
    }
}