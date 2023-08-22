// all of the rusty danda syntax written in one file

    comments
// one liners
/* multiple
    liners */
let danda = /*inside code*/ 1;

    file inclusion
in codeInRustyDanda; // includes code file "./codeInRustyDanda.rd"
lib lib::libInRustyDanda; // includes pre-compiled file "./lib/libInRustyDanda.dnds"

    type declaration
enum Job {
    Programmer,
    Teacher,
    Other,
    None,
}
struct Person {
    age: int,
    name: String,
    job: Job
}
struct Rectangle (float, float, float, float)

    function declaration
fun main(args: Vec<String>){

}