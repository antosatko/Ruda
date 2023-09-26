#![allow(warnings)]

use ast_parser::ast_parser::{generate_ast, Head, HeadParam};
use intermediate::AnalyzationError::ErrType;
use lexer::tokenizer::Tokens;
use lexing_preprocessor::parse_err::Errors;
use std::{env, fs::File, hint::black_box, io::Read, time::SystemTime, collections::HashMap};
use tree_walker::tree_walker::generate_tree;

use crate::{tree_walker::tree_walker::ArgNodeType, intermediate::AnalyzationError};

use lib::*;

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
mod codeblock_parser;
mod lib;

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
            match &parsed_tree {
                Ok(tree) => {
                    println!("Dictionary generated.");
                }
                Err(err) => {
                    println!("Compilation failed.");
                    println!("Errors:");
                    let prepend = format!("Err {}: ", err.1);
                    match &err.0 {
                        ErrorOrigin::LexingError(err) => {
                            for err in err {
                                println!("{prepend}{:?}", err);
                            }
                        }
                        ErrorOrigin::ParsingError(err) => {
                            println!("{prepend}{:?} at {}", err.0, err.1);
                        }
                        ErrorOrigin::CodeBlockParserError(err) => {
                            for err in err {
                                println!("{prepend}{:?}", err);
                            }
                        }
                        ErrorOrigin::IntermediateError(err) => {
                            for err in err {
                                println!("{prepend}{:?}", err);
                            }
                        }
                        ErrorOrigin::LibLoadError(err) => {
                            for err in err {
                                println!("{prepend}{:?}", err);
                            }
                        }
                        ErrorOrigin::LinkingError(err) => {
                            println!("{prepend}{:?}", err);
                        }
                        ErrorOrigin::AnalyzationError(err) => {
                            println!("{prepend}{:?}", err)
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

