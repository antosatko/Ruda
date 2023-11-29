import "#io"
import "#fs"
import "#string"

// import "danda.rd"


struct Danda {
    a: int

    
    new () {
        self.a = 10
        io.print("new" + self.a)
    }

    fun b(self, a: int) {
        io.print(self.a + a)
    }
}

fun main() {
    io.print("hello world");
    let danda = new Danda();
    danda.b(7);
    danda.b(8);
}   

// highlight self