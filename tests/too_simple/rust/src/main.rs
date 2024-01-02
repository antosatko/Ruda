fn main() {
    let mut instant = std::time::Instant::now();
    let mut n = 10000000000f64;
    let mut sum = 0f64;
    let mut flip = -1.0f64;
    let mut i = 1f64;
    while i <= n {
        flip *= -1f64;        
        sum += flip / (2f64*i - 1f64); 
        i += 1f64;
    }
    println!("{:?}", instant.elapsed());
    println!("{}", 4.0 * sum);
}
