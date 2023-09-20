#![allow(warnings)]

use ast_parser::ast_parser::{generate_ast, Head, HeadParam};
use intermediate::AnalyzationError::ErrType;
use lexer::tokenizer::Tokens;
use lexing_preprocessor::parse_err::Errors;
use std::{env, fs::File, hint::black_box, io::Read, time::SystemTime, collections::HashMap};
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
            use lexer::tokenizer::*;
            let ast_path = std::env::var("RUDA_PATH").expect("RUDA_PATH not set.") + "/ruda.ast";
            let mut ast = if let Some(ast) = generate_ast(&ast_path) {
                ast
            } else {
                panic!();
            };
            println!("AST loaded.");
            let parsed_tree = build_dictionaries(&file, &mut (ast.0, ast.1));
            println!("Tree generated.");
            match &parsed_tree {
                Ok(tree) => {
                    println!("Dictionary generated.");
                    // dictionary
                    //println!("{:?}", tree.0);
                    //println!("Imports: {:?}", tree.2);
                }
                Err(err) => {
                    println!("Compilation failed.");
                    println!("Errors:");
                    match &err.0 {
                        ErrorOrigin::LexingError(err) => {
                            for err in err {
                                println!("{:?}", err);
                            }
                        }
                        ErrorOrigin::ParsingError(err) => {
                            for err in err {
                                println!("{:?}", err);
                            }
                        }
                        ErrorOrigin::CodeBlockParserError(err) => {
                            for err in err {
                                println!("{:?}", err);
                            }
                        }
                        ErrorOrigin::IntermediateError(err) => {
                            for err in err {
                                println!("{:?}", err);
                            }
                        }
                        ErrorOrigin::LibLoadError(err) => {
                            for err in err {
                                println!("{:?}", err);
                            }
                        }
                        ErrorOrigin::LinkingError(err) => {
                            println!("{:?}", err);
                        }
                    }
                }
            }
        }
        "tokenize" => {
            let file = match args.nth(0) {
                Some(file) => file,
                None => panic!("File not specified."),
            };
            println!("Tokenization for '{file}' starts.");
            let mut string = String::new();
            let mut file =
                File::open(file).expect(&format!("File not found. ({})", path).to_owned());
            file.read_to_string(&mut string).expect("neco se pokazilo");
            let tokens = tokenize(&string, true);
            println!("Tokens generated.");
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
            libload(&file);
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


pub fn tokenize(content: &str, formating: bool) -> (Vec<Tokens>, Vec<(usize, usize)>, Vec<Errors>) {
    use lexer::tokenizer::*;
    let mut tokens = tokenize(&content.as_bytes(), formating);
    tokens
}

pub enum ErrorOrigin {
    LexingError(Vec<lexing_preprocessor::parse_err::Errors>),
    ParsingError(Vec<lexing_preprocessor::parse_err::Errors>),
    CodeBlockParserError(Vec<lexing_preprocessor::parse_err::Errors>),
    IntermediateError(Vec<lexing_preprocessor::parse_err::Errors>),
    LibLoadError(Vec<lexing_preprocessor::parse_err::Errors>),
    LinkingError(LinkingError),
}

#[derive(Debug)]
pub enum LinkingError {
    /// file, reason
    FileNotFound(String, String),
    /// file, reason
    CouldNotOpen(String, String),
}


pub type Dictionaries = HashMap<String, intermediate::dictionary::Dictionary>;

pub fn read_source(root: &str, main: &str) -> Result<String, ErrorOrigin> {
    let root = std::path::PathBuf::from(root);
    let mut string = String::new();
    let path = root.join(main);
    let mut file = match File::open(&path) {
        Ok(file) => file,
        Err(err) => {
            return Err(ErrorOrigin::LinkingError(LinkingError::FileNotFound(path.to_str().unwrap().to_string(), err.to_string())));
        }
    };
    match file.read_to_string(&mut string) {
        Ok(_) => {},
        Err(err) => {
            return Err(ErrorOrigin::LinkingError(LinkingError::CouldNotOpen(path.to_str().unwrap().to_string(), err.to_string())));
        }
    };
    Ok(string)
}


pub fn new_imports(imports: &mut Vec<String>, new: Vec<String>) {
    imports.extend(new);
    imports.sort();
    imports.dedup();
}

pub fn build_dictionaries(main: &str, ast: &mut (HashMap<String, Head>, Vec<HeadParam>)) -> Result<Dictionaries, (ErrorOrigin, String)> {
    // root is the directory of the main file
    let main_path = std::path::Path::new(main);
    let main = main_path.file_name().expect("internal error 6. please contact the developer.").to_str().expect("internal error 5. please contact the developer.");
    let root = main_path.parent().expect("internal error 0. please contact the developer.").to_str().expect("internal error 0. please contact the developer.");
    let main = match read_source(root, main) {
        Ok(main) => main,
        Err(err) => {
            return Err((err, main.to_string()));
        }
    };
    let mut imports = Vec::new();
    let mut dictionaries = Dictionaries::new();
    match build_dictionary(&main, ast) {
        Ok(res) => {
            if res.1.len() > 0 {
                panic!("internal error 1. please contact the developer.")
            }
            new_imports(&mut imports, res.2);
            dictionaries.insert(main, res.0);
        },
        Err(err) => {
            return Err((err, main.to_string()));
        }
    };
    loop {
        for import in &imports {
            if !dictionaries.contains_key(import) {
                match read_source(root, import) {
                    Ok(main) => {
                        // remove for prod
                        println!("Building {}", import);
                        match build_dictionary(&main, ast) {
                            Ok(res) => {
                                if res.1.len() > 0 {
                                    panic!("internal error 2. please contact the developer.")
                                }
                                new_imports(&mut imports.clone(), res.2);
                                dictionaries.insert(main, res.0);
                            },
                            Err(err) => {
                                return Err((err, main.to_string()));
                            }
                        };
                    },
                    Err(err) => {
                        return Err((err, import.to_string()));
                    }
                };
            }
        }
        // check if all imports are in the dictionary
        let mut all = true;
        for import in &imports {
            if !dictionaries.contains_key(import) {
                all = false;
                break;
            }
        }
        if all {
            return Ok(dictionaries);
        }
    }

    
}


/// you cannot kill me in a way that matters
pub fn build_dictionary(mut content: &str, ast: &mut (HashMap<String, ast_parser::ast_parser::Head>, Vec<ast_parser::ast_parser::HeadParam>)) -> Result<(intermediate::dictionary::Dictionary, Vec<ErrType>, Vec<String>), ErrorOrigin> {
    let mut tokens = tokenize(&content, false);
    if tokens.2.len() > 0 {
        return Err(ErrorOrigin::LexingError(tokens.2));
    }
    tokens.0 = if let Ok(toks) = lexing_preprocessor::lexing_preprocessor::refactor(
        tokens.0,
        tokens.1,
        &mut tokens.2,
    ) {
        tokens.1 = toks.1;
        toks.0
    } else {
        return Err(ErrorOrigin::LexingError(tokens.2));
    }; //tokenize(&string, true);
    if tokens.2.len() > 0 {
        return Err(ErrorOrigin::LexingError(tokens.2));
    }
    let parsed_tree = generate_tree(&tokens.0, ast, &tokens.1);
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
            
            // println!("Imports: {:?}", imports);

            let mut dictionary = intermediate::dictionary::from_ast(&tree.nodes, &imports);
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
            return Ok((dictionary.0, dictionary.1, imports));
        }
        None => {
            return Err(ErrorOrigin::ParsingError(tokens.2));
        }
    }
}

pub fn libload(file: &str) -> Result<libloader::Dictionary, String> {
    let lib = unsafe { libloading::Library::new(file).expect("Failed to load library.") };
    let register = unsafe {
        lib.get::<fn()->String>(b"register\0")
            .expect("Failed to load register function.")
    }();
    let lib = libloader::load(&register.as_bytes());
    println!("Library loaded.");
    println!("Library: {:#?}", lib);
    lib
}