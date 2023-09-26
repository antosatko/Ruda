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
    let bin_paths = profile.1.binaries.iter().map(|(_, path)|path.clone()).collect();
    let mut binaries = build_binaries(&bin_paths);
    println!("Dictionary generated.");
    println!("{:?}", dictionaries);
    println!("Binaries generated.");
    for (name, binary) in &profile.1.binaries {
        println!("{name}");
        println!("{binary}");
    }
    // TODO: uncomment for prod
    // sum::write_sums(path, profile.0, &sum::sum(path, profile.0));
}
