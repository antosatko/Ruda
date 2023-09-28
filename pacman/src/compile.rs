use std::collections::HashMap;

use crate::{config, sum};

pub fn compile(path: &str, profile: (&str, &config::Profile)) {
    // determine if we have to compile for current profile
    let mut compile = false;
    // check if there is directory for the profile
    let profile_path = std::path::Path::new(path).join("target").join(profile.0);
    if !profile_path.exists() {
        compile = true;
        // create directory
        std::fs::create_dir_all(&profile_path).unwrap();
    }
    // check if there is a sums.txt file
    let sums_path = profile_path.join(sum::TARGET_FILE);
    if !sums_path.exists() {
        compile = true;
    } else if !sum::check(path, profile.0) {
        compile = true;
    }
    if !compile {
        return;
    }
    // compile
    let ruda_path = match std::env::var("RUDA_PATH") {
        Ok(path) => path,
        Err(err) => {
            println!("RUDA_PATH not found. {}\nProject not compiled.", err);
            return;
        }
    };
    let bin_kind = String::from("src/");
    let main_file = match profile.1.kind {
        config::ProjectKind::Bin => {
            let bin_path = std::path::Path::new(path).join("src").join("main.rd");
            bin_path
        }
        config::ProjectKind::Lib => {
            let lib_path = std::path::Path::new(path).join("src").join("lib.rd");
            lib_path
        }
    };
    let main_file = match main_file.to_str() {
        Some(file) => file,
        None => {
            println!("Failed to convert path to string.");
            return;
        }
    };
    println!("Compiling... {} {}", path, profile.0);
    use compiler::*;
    let (ast, params, registry) = match generate_ast(&ruda_path) {
        Ok(ast) => (ast.ast, ast.params, ast.registry),
        Err(err) => {
            println!("Failed to load AST.");
            println!("{}", err);
            return;
        }
    };
    let mut dictionaries = build_dictionaries(&main_file, &mut (ast, params));
    println!("Dictionary generated.");
    println!("{:?}", dictionaries);
    let mut bin_paths = Vec::new();
    let mut lib_names = Vec::new();
    for (lib_name, lib_path) in &profile.1.binaries {
        let lib_path = std::path::Path::new(path).join(lib_path);
        if !lib_path.exists() {
            println!("{} does not exist.", lib_path.to_str().unwrap());
            return;
        }
        let lib_path = match lib_path.to_str() {
            Some(path) => path,
            None => {
                println!("Failed to convert path to string.");
                return;
            }
        };
        bin_paths.push(lib_path.to_string());
        lib_names.push(lib_name.to_string());
    }
    let mut binaries = match build_binaries(&bin_paths) {
        Ok(binaries) => binaries,
        Err(err) => {
            println!("Failed to load binaries.");
            println!("{}", err);
            return;
        }
    };
    let mut binaries = {
        let mut bins = HashMap::new();
        for (lib_name, lib_path) in lib_names.iter().zip(binaries.iter_mut()) {
            bins.insert(lib_name.to_string(), lib_path);
        }
        bins
    };
    println!("Binaries generated.");
    println!("{:?}", binaries);
    // TODO: uncomment for prod
    // sum::write_sums(path, profile.0, &sum::sum(path, profile.0));
}
