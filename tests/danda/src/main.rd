import "#io"
import "#fs"
import "#string"

// import "danda.rd"


struct Danda {
    a: &int


    new () {
        self.a = new 0
    }

    fun b() {
        io.println("Hello World")
    }
}

fun main() {
    let danda = new Danda()
    
    danda.b()
}   
