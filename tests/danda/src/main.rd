import "#io"
import "#fs"
import "#string"

import "danda.rd"


fun main() {
    /*let a: danda.Danda = new danda.Danda(6)

    danda.a = 10    */

    let a: &int = new 5;

    *a = string.parse("10");

    io.println(*a);
}   
