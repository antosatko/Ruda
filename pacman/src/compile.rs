use std::{collections::HashMap, process::id};

use crate::{config::{self, Runtime}, sum};

use compiler::prep_objects::Context;

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
            println!("{}", err);
            println!("Close all programs that use Ruda and try again.");
            println!("If that doesn't help, try to reinstall Ruda.");
            return;
        }
    };
    println!("AST generated.");
    let dictionaries = match build_dictionaries(&main_file, &mut (ast, params)) {
        Ok(dictionaries) => dictionaries,
        Err(err) => {
            println!("Failed to load dictionaries.");
            println!("Err: '{}':{}", err.1, err.0);
            return;
        }
    };
    println!("Dictionary generated.");
    println!("{:?}", dictionaries);
    // BEWARE: this part is what you call a technical debt
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
    let mut temp_ast = (registry, Vec::new());
    let mut binaries = HashMap::new();
    let mut std_lib = match build_std_lib(&mut temp_ast) {
        Ok(std_lib) => std_lib,
        Err(err) => {
            println!("Failed to load std lib.");
            println!("{}", err);
            return;
        }
    };
    let mut dicts = Vec::new();
    let mut names = Vec::new();
    for _ in 0..std_lib.len() {
        let take = std_lib.remove(0);
        dicts.push(take.0);
        names.push(take.1);
    }
    drop(std_lib);
    match build_binaries(&bin_paths, &mut temp_ast, &mut dicts) {
        Ok(()) => {},
        Err(err) => {
            println!("Failed to load binaries.");
            println!("{}", err);
            return;
        }
    };
    for _ in 0..names.len() {
        binaries.insert(names.remove(0), dicts.remove(0));
    }
    println!("{:?}", binaries.keys());
    for (_, libname) in lib_names.iter().enumerate() {
        binaries.insert(libname.to_string(), dicts.remove(0));
    }
    const LIB_COUNT: usize = 4;
    const STD_LIBS: [&str; LIB_COUNT] = ["#io", "#string", "#fs", "#algo"];
    let mut count = LIB_COUNT;
    for (name, bin) in binaries.iter_mut() {
        match STD_LIBS.iter().position(|&lib| lib == name) {
            Some(idx) => {
                bin.id = idx;
            }
            None => {
                bin.id = count;
                count += 1;
            }
        }
        
    }
    println!("Binaries generated.");
    println!("{:?}", dicts);
    let mut context = Context::new(dictionaries, binaries);
    match prep_objects::prep(&mut context) {
        Ok(_) => {
            println!("Objects prepared.");
            println!("{:?}", context.destruct());
        }
        Err(err) => {
            println!("Failed to prepare objects.");
            // TODO: println!("{}", err);
            return;
        }
    }

    let executable = match codegen::gen(&mut context, "main.rd") {
        Ok(ctx) => {
            println!("{:?}", ctx.code.data);
            println!("{:?}", ctx.code.entry_point);
            println!("{:?}", ctx.memory.heap.data);
            println!("{:?}", ctx.memory.stack.data);
            println!("{:?}", ctx.memory.strings.pool);
            println!("{:?}", ctx.memory.non_primitives);

            let code = codegen::stringify(&ctx, &Vec::new());
            code
        }
        Err(err) => {
            println!("Failed to generate code: {:?}", err);
            return;
        }
    };


    {
        let mut path = std::path::Path::new(path).join("target").join(profile.0);
        path = path.join("out.rdbin");
        std::fs::write(path, executable).unwrap();
    }

    // TODO: uncomment for prod
    sum::write_sums(path, profile.0, &sum::sum(path, profile.0));
}
