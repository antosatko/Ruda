import "#io"
import "#fs"
import "#string"
import "#time"
import "#window" as win
import "#math"

struct A {
    a: int
    b: int
    node: A?

    new(a: int, b: int) {
        self.a = a
        self.b = b
    }

    fun add(self, num: int) {
        self.a += num
        self.b += num
    }

    fun append(self, node: A) {
        self.node = node
    }

    fun print(self) {
        io.println("A(" + self.a + ", " + self.b + ")")
    }
}


fun main() {
    let a = A(1, 2)
    a.add(3)

    io.println(a.a)
    io.println(a.b)

    a.append(A(3, 4))


    
    if a.node? {
        io.println("node exists")
        a.node.print()
        a.node.append(A(5, 6))
        a.node.node.print()
    }else {
        io.println("node doesn't exist")
        io.println(null)
    }
}