use intermediate::dictionary;

use crate::ast_parser::ast_parser::{generate_ast as gen_ast, Head, HeadParam};
use crate::intermediate::AnalyzationError::ErrType;
use crate::lexer::tokenizer::Tokens;
use crate::lexing_preprocessor::parse_err::Errors;
use crate::tree_walker::tree_walker::generate_tree;
use std::path::Path;
use std::{collections::HashMap, env, fs::File, hint::black_box, io::Read, time::SystemTime};

use crate::{intermediate::AnalyzationError, tree_walker::tree_walker::ArgNodeType};

mod ast_parser;
mod lexer;
//mod reader;
extern crate runtime;
mod lexing_preprocessor;
mod tree_walker;
//mod writer;
mod codeblock_parser;
mod expression_parser;
mod intermediate;
mod libloader;
pub mod prep_objects;
pub mod codegen;

pub fn tokenize(content: &str, formating: bool) -> (Vec<Tokens>, Vec<(usize, usize)>, Vec<Errors>) {
    use lexer::tokenizer::*;
    let mut tokens = tokenize(&content.as_bytes(), formating);
    tokens
}

#[derive(Debug)]
pub enum ErrorOrigin {
    LexingError(Vec<lexing_preprocessor::parse_err::Errors>),
    ParsingError(
        (
            tree_walker::tree_walker::Err,
            tree_walker::tree_walker::Line,
        ),
    ),
    CodeBlockParserError(Vec<lexing_preprocessor::parse_err::Errors>),
    IntermediateError(Vec<lexing_preprocessor::parse_err::Errors>),
    LibLoadError(Vec<lexing_preprocessor::parse_err::Errors>),
    AnalyzationError(Vec<intermediate::AnalyzationError::ErrType>),
    LinkingError(LinkingError),
}

impl std::fmt::Display for ErrorOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorOrigin::LexingError(errs) => {
                write!(f, "Lexing error:\n")?;
                for err in errs {
                    write!(f, "{}\n", err)?;
                }
                Ok(())
            }
            ErrorOrigin::ParsingError((err, line)) => {
                write!(f, "Parsing error:\n")?;
                write!(f, "{}\n", err)?;
                write!(f, "Line: {}", line)?;
                Ok(())
            }
            ErrorOrigin::CodeBlockParserError(errs) => {
                write!(f, "Code block parsing error:\n")?;
                for err in errs {
                    write!(f, "{}\n", err)?;
                }
                Ok(())
            }
            ErrorOrigin::IntermediateError(errs) => {
                write!(f, "Intermediate error:\n")?;
                for err in errs {
                    write!(f, "{}\n", err)?;
                }
                Ok(())
            }
            ErrorOrigin::LibLoadError(errs) => {
                write!(f, "Library loading error:\n")?;
                for err in errs {
                    write!(f, "{}\n", err)?;
                }
                Ok(())
            }
            ErrorOrigin::AnalyzationError(errs) => {
                write!(f, "Analyzation error:\n")?;
                for err in errs {
                    write!(f, "{}\n", err)?;
                }
                Ok(())
            }
            ErrorOrigin::LinkingError(err) => {
                write!(f, "Linking error:\n")?;
                write!(f, "{:?}", err)?;
                Ok(())
            }
        }
    }
}

#[derive(Debug)]
pub enum LinkingError {
    /// file, reason
    FileNotFound(String, String),
    /// file, reason
    CouldNotOpen(String, String),
}

pub type Dictionaries = HashMap<String, intermediate::dictionary::Dictionary>;

pub fn read_source(root: &str, main: &str) -> Result<Option<String>, ErrorOrigin> {
    if main.starts_with("#") {
        return Ok(None);
    }
    let root = std::path::PathBuf::from(root);
    let mut string = String::new();
    let path = root.join(main);
    let mut file = match File::open(&path) {
        Ok(file) => file,
        Err(err) => {
            return Err(ErrorOrigin::LinkingError(LinkingError::FileNotFound(
                path.to_str().unwrap().to_string(),
                err.to_string(),
            )));
        }
    };
    match file.read_to_string(&mut string) {
        Ok(_) => {}
        Err(err) => {
            return Err(ErrorOrigin::LinkingError(LinkingError::CouldNotOpen(
                path.to_str().unwrap().to_string(),
                err.to_string(),
            )));
        }
    };
    Ok(Some(string))
}

pub fn new_imports(imports: &mut Vec<String>, new: Vec<String>) {
    imports.extend(new);
    imports.sort();
    imports.dedup();
}

pub fn build_dictionaries(
    main: &str,
    ast: &mut (HashMap<String, Head>, Vec<HeadParam>),
) -> Result<Dictionaries, (ErrorOrigin, String)> {
    // root is the directory of the main file
    let main_path = std::path::Path::new(main);
    let main_ = main_path
        .file_name()
        .expect("internal error 6. please contact the developer.")
        .to_str()
        .expect("internal error 5. please contact the developer.");
    let root = main_path
        .parent()
        .expect("internal error 0. please contact the developer.")
        .to_str()
        .expect("internal error 0. please contact the developer.");
    let main = match read_source(root, main_) {
        Ok(main) => main.unwrap(),
        Err(err) => {
            return Err((err, main_.to_string()));
        }
    };
    let mut imports = Vec::new();
    let mut dictionaries = Dictionaries::new();
    match build_dictionary(&main, ast, main_) {
        Ok(res) => {
            if res.1.len() > 0 {
                panic!("internal error 1. please contact the developer.")
            }
            new_imports(&mut imports, res.2);
            dictionaries.insert(main_.to_string(), res.0);
        }
        Err(err) => {
            return Err((err, root.to_string()));
        }
    };
    let mut i = 0;
    while i < imports.len() {
        if imports[i].starts_with("#") {
            imports.remove(i);
        } else {
            i += 1;
        }
    }
    loop {
        let mut found_imports = Vec::new();
        for import in &imports {
            if !dictionaries.contains_key(import) {
                match read_source(root, import) {
                    Ok(main) => {
                        if main.is_none() {
                            found_imports.push(import.clone());
                            continue;
                        }
                        match build_dictionary(&main.unwrap(), ast, import) {
                            Ok(res) => {
                                if res.1.len() > 0 {
                                    return Err((
                                        ErrorOrigin::AnalyzationError(res.1),
                                        import.clone(),
                                    ));
                                }
                                found_imports.extend(res.2);
                                dictionaries.insert(import.clone(), res.0);
                            }
                            Err(err) => {
                                return Err((err, import.to_string()));
                            }
                        };
                    }
                    Err(err) => {
                        return Err((err, import.to_string()));
                    }
                };
            }
        }
        let mut i = 0;
        while i < found_imports.len() {
            if found_imports[i].starts_with("#") {
                found_imports.remove(i);
            } else {
                i += 1;
            }
        }
        new_imports(&mut imports, found_imports);
        // check if all imports are in the dictionary
        let mut all = true;
        for import in &imports {
            if !dictionaries.contains_key(import) {
                all = false;
                break;
            }
        }
        if all {
            // for dict in &dictionaries {
            //     println!("Dictionary: {}", dict.0);
            //     println!("{:?}", dict.1);
            // }
            return Ok(dictionaries);
        }
    }
}

/// returns all of the binaries as dictionaries in the order of the paths
pub fn build_binaries(
    paths: &Vec<String>,
    ast: &mut (HashMap<String, Head>, Vec<HeadParam>),
    binaries: &mut Vec<libloader::Dictionary>,
) -> Result<(), String> {
    for path in paths {
        const SUFFIXES: [&str; 2] = [".dll", ".so"];
        let mut name = Path::new(path).file_name().unwrap().to_str().unwrap().to_string();
        for suffix in &SUFFIXES {
            if name.ends_with(suffix) {
                name = name.strip_suffix(suffix).unwrap().to_string();
                break;
            }
        }
        binaries.push(libload(&path, ast, &name)?);
    }
    Ok(())
}

pub fn build_std_lib(ast: &mut (HashMap<String, Head>, Vec<HeadParam>)) -> Result<Vec<(libloader::Dictionary, String)>, String> {
    let mut binaries = Vec::new();
    let mut path = env::var("RUDA_PATH").unwrap_or_else(|_| ".".to_string());
    path.push_str("/stdlib");
    if !std::path::Path::new(&path).exists() {
        return Err(format!("Could not find stdlib at '{}'.", path));
    }
    let path = std::path::PathBuf::from(path);
    let dir = match std::fs::read_dir(path) {
        Ok(dir) => dir,
        Err(err) => {
            return Err(format!("Could not read stdlib directory. {}", err));
        }
    };
    for file in dir {
        let file = match file {
            Ok(file) => file,
            Err(err) => {
                return Err(format!("Could not read stdlib directory. {}", err));
            }
        };
        let path = file.path();
        let path = match path.to_str() {
            Some(path) => path,
            None => {
                return Err(format!("Could not read stdlib directory."));
            }
        };
        let filename = match file.file_name().to_str() {
            Some(name) => {
                let mut tmp = String::from("#");
                tmp.push_str(name.strip_suffix(".dll").unwrap());
                tmp
            }
            None => {
                return Err(format!("Could not read stdlib directory."));
            }
        };
        const SUFFIXES: [&str; 2] = [".dll", ".so"];
        let mut name = Path::new(path).file_name().unwrap().to_str().unwrap().to_string();
        for suffix in &SUFFIXES {
            if name.ends_with(suffix) {
                name = name.strip_suffix(suffix).unwrap().to_string();
                break;
            }
        }
        let lib = libload(path, ast, &name)?;
        binaries.push((lib, filename));
    }

    Ok(binaries)
}

/// you cannot kill me in a way that matters
pub fn build_dictionary(
    mut content: &str,
    ast: &mut (
        HashMap<String, ast_parser::ast_parser::Head>,
        Vec<ast_parser::ast_parser::HeadParam>,
    ),
    file_name: &str,
) -> Result<
    (
        intermediate::dictionary::Dictionary,
        Vec<ErrType>,
        Vec<String>,
    ),
    ErrorOrigin,
> {
    let mut tokens = tokenize(&content, false);
    if tokens.2.len() > 0 {
        return Err(ErrorOrigin::LexingError(tokens.2));
    }
    tokens.0 = if let Ok(toks) =
        lexing_preprocessor::lexing_preprocessor::refactor(tokens.0, tokens.1, &mut tokens.2)
    {
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
        Ok((tree, globals)) => {
            let mut imports = Vec::new();
            if let ArgNodeType::Array(arr) = globals.get("imports").unwrap() {
                for global in arr {
                    if let Tokens::String(str) = &global.name {
                        imports.push(str.to_string());
                    }
                }
            }

            // println!("Imports: {:?}", imports);

            let mut dictionary = intermediate::dictionary::from_ast(&tree.nodes, &imports, file_name);
            return Ok((dictionary.0, dictionary.1, imports));
        }
        Err(err) => {
            return Err(ErrorOrigin::ParsingError(err.clone()));
        }
    }
}

pub fn libload(
    file: &str,
    ast: &mut (HashMap<String, Head>, Vec<HeadParam>),
    file_identifier: &str,
) -> Result<libloader::Dictionary, String> {
    let lib = unsafe {
        match libloading::Library::new(file) {
            Ok(lib) => lib,
            Err(err) => {
                return Err(format!("Failed to load library '{file}'. {}", err));
            }
        }
    };
    let register = unsafe {
        match lib.get::<fn() -> String>(b"register\0") {
            Ok(register) => register,
            Err(err) => {
                return Err(format!("Library is not correct format '{file}'. {}", err));
            }
        }
    }();
    let lib = libloader::load(&register.as_bytes(), ast, file_identifier);
    lib
}

pub fn generate_ast(ruda_path: &str) -> Result<Asts, AstGenError> {
    use lexer::tokenizer::*;
    // ruda_path + "/ruda.ast"
    let ast_path = std::path::PathBuf::from(ruda_path).join("ruda.ast");
    let ast_path = match ast_path.to_str() {
        Some(path) => path,
        None => return Err(AstGenError::NotFound(AstType::Ast)),
    };
    let (ast, globals) = if let Some(ast) = gen_ast(&ast_path) {
        ast
    } else {
        return Err(AstGenError::CouldNotOpen(AstType::Ast));
    };
    let registry_path = std::path::PathBuf::from(ruda_path).join("registry.ast");
    let registry_path = match registry_path.to_str() {
        Some(path) => path,
        None => return Err(AstGenError::NotFound(AstType::Registry)),
    };
    let (registry, _) = if let Some(ast) = gen_ast(&registry_path) {
        ast
    } else {
        return Err(AstGenError::CouldNotOpen(AstType::Registry));
    };
    return Ok(Asts {
        ast,
        params: globals,
        registry,
    });
}

#[derive(Debug)]
pub enum AstGenError {
    NotFound(AstType),
    ParseError(AstType),
    CouldNotOpen(AstType),
}

impl std::fmt::Display for AstGenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AstGenError::NotFound(ast) => write!(f, "Could not find {} file.", ast),
            AstGenError::ParseError(ast) => write!(f, "Could not parse {} file.", ast),
            AstGenError::CouldNotOpen(ast) => write!(f, "Could not open {} file.", ast),
        }
    }
}

#[derive(Debug)]
pub enum AstType {
    Ast,
    Registry,
}

impl std::fmt::Display for AstType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AstType::Ast => write!(f, "ast"),
            AstType::Registry => write!(f, "registry"),
        }
    }
}

pub struct Asts {
    pub ast: HashMap<String, Head>,
    pub params: Vec<HeadParam>,
    pub registry: HashMap<String, Head>,
}
