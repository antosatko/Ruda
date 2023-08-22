#![allow(warnings)]

use ast_parser::ast_parser::generate_ast;
use std::{env, fs::File, hint::black_box, io::Read, time::SystemTime};
use tree_walker::tree_walker::generate_tree;

use crate::tree_walker::tree_walker::ArgNodeType;

mod ast_parser;
mod lexer;
//mod reader;
extern crate runtime;
mod lexing_preprocessor;
mod tree_walker;
//mod writer;
mod expression_parser;
mod intermediate;
mod libloader;
mod type_check;
mod codeblock_parser;

fn main() {
    let mut args = env::args();
    let path = match args.nth(0) {
        Some(path) => path,
        None => panic!("Path not specified."),
    };
    let cmd = match args.nth(0) {
        Some(cmd) => cmd,
        None => String::from(""),
    };

    match cmd.as_str() {
        "build" => {
            let file = match args.nth(0) {
                Some(file) => file,
                None => panic!("File not specified."),
            };
            println!("Compilation for '{file}' starts.");
            let mut string = String::new();
            let mut file =
                File::open(file).expect(&format!("File not found. ({})", path).to_owned());
            file.read_to_string(&mut string).expect("neco se pokazilo");
            let string = string.into_bytes();
            use lexer::tokenizer::*;
            let ast_path = std::env::var("RUDA_PATH").expect("RUDA_PATH not set.") + "/ruda.ast";
            let mut ast = if let Some(ast) = generate_ast(&ast_path) {
                ast
            } else {
                panic!();
            };
            println!("AST loaded.");
            let time = SystemTime::now();
            let mut tokens = tokenize(&string, false);
            tokens.0 = if let Ok(toks) = lexing_preprocessor::lexing_preprocessor::refactor(
                tokens.0,
                tokens.1,
                &mut tokens.2,
            ) {
                tokens.1 = toks.1;
                toks.0
            } else {
                panic!("hruzostrasna pohroma");
            }; //tokenize(&string, true);
            println!("Tokens generated. {:?}", tokens.0);
            if tokens.2.len() > 0 {
                println!("Compilation failed.");
                println!("Errors:");
                for err in tokens.2 {
                    println!("{:?}", err);
                }
                return;
            }
            let parsed_tree = generate_tree(&tokens.0, (&ast.0, &mut ast.1), &tokens.1);
            println!("AST generated.");
            println!(
                "time: {}",
                SystemTime::now().duration_since(time).unwrap().as_millis()
            );
            use intermediate::dictionary;
            match &parsed_tree {
                Some((tree, globals)) => {
                    let mut imports = Vec::new();
                    if let ArgNodeType::Array(arr) = globals.get("imports").unwrap() {
                        for global in arr {
                            if let Tokens::String(str) = &global.name {
                                imports.push(str.to_string());
                            }
                        }
                    }
                    println!("Imports: {:?}", imports);
                    dictionary::from_ast(&tree.nodes, &imports);
                    println!("Dictionary generated.");
                    println!(
                        "time: {}",
                        SystemTime::now().duration_since(time).unwrap().as_millis()
                    );
                    black_box(tree);
                }
                None => {
                    println!("Aborting.");
                    return;
                }
            }
            println!("Parsed.");
            println!(
                "time: {}",
                SystemTime::now().duration_since(time).unwrap().as_millis()
            );
            if false {
                if let Some(nodes) = &parsed_tree {
                    use tree_walker::tree_walker::ArgNodeType;
                    for nod in &nodes.0.nodes {
                        println!("{:?}", nod.0);
                        match nod.1 {
                            ArgNodeType::Array(arr) => {
                                for arg in arr {
                                    println!("{arg:?}");
                                }
                            }
                            ArgNodeType::Value(val) => {
                                println!("{val:?}");
                            }
                        }
                    }
                }
            }
            black_box(parsed_tree);
        }
        "tokenize" => {
            let file = match args.nth(0) {
                Some(file) => file,
                None => panic!("File not specified."),
            };
            println!("Compilation for '{file}' starts.");
            let mut string = String::new();
            let mut file =
                File::open(file).expect(&format!("File not found. ({})", path).to_owned());
            file.read_to_string(&mut string).expect("neco se pokazilo");
            let string = string.into_bytes();
            use lexer::tokenizer::*;
            let tokens = tokenize(&string, true);
            println!("{:?}", tokens.0);
        }
        "astTest" => {
            let mut file_name = String::from("ast/");
            match args.nth(0) {
                Some(file) => file_name.push_str(&file),
                None => {
                    println!("file not specified");
                    return;
                }
            };
            if let Some(ast) = generate_ast(&file_name) {
                for node in ast.0 {
                    println!("{:?}\n", node)
                }
            } else {
                println!("failed to parse AST properly")
            }
        }
        "libload" => {
            let file = match args.nth(0) {
                Some(file) => file,
                None => panic!("File not specified."),
            };
            println!("Loading library '{file}' starts.");
            let mut string = String::new();
            let mut file =
                File::open(file).expect(&format!("File not found. ({})", path).to_owned());
            file.read_to_string(&mut string).expect("neco se pokazilo");
            libloader::load(&mut string.into_bytes());
        }
        "test" => {
            use std::path::Path;
            use std::ffi::OsStr;
            use std::env;

            let path = Path::new("./ahoj.txt");
            println!("Absolute path: {:?}", env::current_dir().unwrap().join(path));

            println!("Path: {:?}", path);
            println!("Extension: {:?}", path.extension());
            println!("File name: {:?}", path.file_name());
            println!("Parent directory: {:?}", path.parent());
            // if is file
            if path.is_file() {
                println!("File exists");
                // if is txt file
                if path.extension() == Some(OsStr::new("txt")) {
                    println!("Extension is txt");
                    // print content
                    let str = String::from_utf8(std::fs::read(path).unwrap()).unwrap();
                    println!("Content: {}", String::from_utf8(std::fs::read(path).unwrap()).unwrap());
                }
            }else {
                println!("File does not exist");
            }
        }
        "help" => {
            let msg = r#"This is a compiler for the language Rusty Danda.

Usage: {} [command] [args]
Commands:
    build [file] - compiles file - not implemented yet
    tokenize [file] - prints tokens of file
    astTest [file] - tests if AST can be loaded properly, if not, you will get an error hopefully
                     also if you get an infinite loop, it means that one or more of the AST nodes
                     are not terminated properly (missing semicolon)
    libload [file] - tests if library can be loaded properly, if not, you will get an error hopefully
    help - shows this message
            "#;
            println!("{msg}");
        }
        _ => {
            println!("Unknown command: {}", cmd);
            println!("Try help.");
        }
    }
}
