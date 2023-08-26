use std::io::{Read, Write};
use sha2::Digest;
use sha2;


pub const TARGET_FILE: &str = "sums.txt";

/// Sum all files in the src directory
/// returns a vector of sums
/// 
/// uses sha256 and sha512
pub fn sum(path: &str, profile: &str) -> Vec<String> {
    let mut sums = Vec::new();
    let file = std::path::Path::new(path);
    // check if directory target/ exists, if not create it
    let target = file.join("target").join(profile);
    if !target.exists() {
        std::fs::create_dir_all(&target).unwrap();
    }
    // check if file target/sums.txt exists, if not create it
    let target_file = target.join(TARGET_FILE);
    if !target_file.exists() {
        std::fs::File::create(&target_file).unwrap();
    }
    // read all files in src/
    let files = std::fs::read_dir(file.join("src")).unwrap();
    for file in files {
        let file = file.unwrap();
        let path = file.path();
        // check if file is a directory
        if path.is_dir() {
            continue;
        }
        // check if file is target/sums.txt
        if path == target_file {
            continue;
        }
        // read file
        let mut file = std::fs::File::open(&path).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        // calculate sums
        let mut sha256 = sha2::Sha256::new();
        sha256.update(&buffer);
        let mut sha512 = sha2::Sha512::new();
        sha512.update(&buffer);
        // push sums to vector
        sums.push(format!("{:x} {:x} {}", sha256.finalize(), sha512.finalize(), path.display()));
    }
    sums
}


/// write sums to target/sums.txt
/// path is the path to the project
pub fn write_sums(path: &str, profile: &str, sums: &Vec<String>) {
    let target = std::path::Path::new(path).join("target").join(profile);
    let target_file = target.join(TARGET_FILE);
    let mut file = std::fs::File::create(&target_file).unwrap();
    for sum in sums {
        file.write_all(sum.as_bytes()).unwrap();
        file.write_all(b"\n").unwrap();
    }
}


/// check if sums are correct
/// path is the path to the project
pub fn check(path: &str, profile: &str) -> bool {
    let target = std::path::Path::new(path).join("target").join(profile);
    let target_file = target.join(TARGET_FILE);
    if !target_file.exists() {
        return false;
    }
    let sums = sum(path, profile);
    let mut file = std::fs::File::open(&target_file).unwrap();
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).unwrap();
    let mut sums_file = Vec::new();
    for line in buffer.lines() {
        sums_file.push(line.to_string());
    }
    sums == sums_file
}