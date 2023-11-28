import "#io"
import "#fs"
import "#string"

// import "danda.rd"


struct Danda {
    a: &int

    
    new () {
        self.a = new 0
    }
}

fun main() {
    let danda = Danda()

    io.println(*danda.a);

    *danda.a = 10

    let a: int = *danda.a

    io.println(a)
}   
