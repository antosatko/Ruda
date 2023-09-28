use std::{env, fs::File, io::Read};

use compiler::*;

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
            let ruda_path = std::env::var("RUDA_PATH").expect("RUDA_PATH not set.");
            let (ast, params, registry) = match generate_ast(&ruda_path) {
                Ok(ast) => (ast.ast, ast.params, ast.registry),
                Err(err) => {
                    println!("Failed to load AST.");
                    println!("{}", err);
                    return;
                }
            };

            println!("AST loaded.");
            let parsed_tree = build_dictionaries(&file, &mut (ast, params));
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
            let ruda_path = std::env::var("RUDA_PATH").expect("RUDA_PATH not set.");
            if let Ok(ast) = generate_ast(&ruda_path) {
                println!("AST loaded.");
                println!("{:#?}", ast.ast);
            } else {
                println!("Failed to load AST.");
            }
        }
        "libload" => {
            let file = match args.nth(0) {
                Some(file) => file,
                None => panic!("File not specified."),
            };
            let mute = match args.nth(0) {
                Some(mute) => mute == "mute",
                None => false,
            };
            println!("Loading library '{file}' starts.");
            match libload(&file) {
                Ok(lib) => {
                    if !mute {
                        println!("Library loaded.");
                        println!("{:#?}", lib);
                    }
                    std::process::exit(0);
                }
                Err(err) => {
                    println!("Failed to load library.");
                    println!("{}", err);
                    std::process::exit(1);
                }
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

