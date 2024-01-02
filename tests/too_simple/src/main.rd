import "#io"


// Hello curious reader! This is a simple program that calculates pi using the Leibniz formula. (I stole it from the internet)
//
// I will use this file to compare the performance of different languages.

// Benchmark results:
// - Rust (release): 12.8678 seconds
// - Rust (debug): 27.459 seconds 
// - Ruda: 31.27 minutes (this is before optimizations)
// - Python (while loop): I turned it off after 2 hours

fun main() {
    let n = 10000000000f
    let sum = 0f
    let flip = -1f
    let i = 1f
    while i <= n {
        flip *= -1f
        sum += flip / (2f*i - 1f)
        i += 1f
    }
    io.println(sum*4f)
}