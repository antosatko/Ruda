import "#io"


struct Danda {
    a: int
    e: int

    
    new(a: &int) {
        self.e = a
        io.println(self.e)
    }
}