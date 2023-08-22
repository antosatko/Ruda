/*
    This is my playground where I test new compiler features
*/
use super::print;

enum Dandove {
    Jeden;
    Druhej;;;;;;;
    Patej = 5;
}

// structs are basically classes from js, i just like to call it struct instead of class
struct Cislo {
    nevim: int;
    fun new (num): Self {
        Self {
            nevim: num;
        }
    }
}

// entry point of your program
fun main(): int {
    let danda = 123.54;
    if danda == 50f {
        danda += 50f;
        print("tohle je fakt /*cool*/ string");
    }
    else {
        print("Hello,\n World!");
    }
    yeet 0;
}