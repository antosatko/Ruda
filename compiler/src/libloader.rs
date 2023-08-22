use std::collections::HashMap;

use crate::{intermediate::{self, AnalyzationError::ErrType}, lexer::tokenizer::{Tokens, self, Operators}, ast_parser::ast_parser::generate_ast, tree_walker::tree_walker::{generate_tree, Node, Err}, lexing_preprocessor::{lexing_preprocessor, parse_err::Errors}, expression_parser::{ValueType, get_args}};
use intermediate::dictionary::*;
use lexing_preprocessor::*;


pub fn load(string: &mut Vec<u8>) -> Result<Dictionary, String> {
    let (mut tokens, mut lines, mut errs) = tokenizer::tokenize(string, true);
    let ast_path = std::env::var("RUDA_PATH").unwrap() + "/registry.ast";
    let ast = if let Some(ast) = generate_ast(&ast_path) {
        ast
    } else {
        return Err("".to_owned());
    };
    let tree = if let Some(tree) = generate_tree(&tokens, (&ast.0, &mut vec![]), &lines) {
        println!("{:?}", tree);
        tree
    } else {
        return Err("".to_owned());
    };
    let mut errors = Vec::new();
    let mut dictionary = Dictionary::new();
    for node in step_inside_arr(&tree.0, "nodes") {
        if let Tokens::Text(name) = &node.name {
            match name.as_str() {
                "KWStruct" => {
                    let ident = get_ident(&node);
                    let generics = get_generics_decl(&node, &mut errors);
                    let assign = get_assign(&node);
                    let mut fields: Vec<(String, ShallowType)> = Vec::new();
                    for key in step_inside_arr(node, "keys") {
                        let ident = get_ident(&key);
                        for field in &fields {
                            if *field.0 == ident {
                                errors.push(ErrType::StructVariantAssignedIdent(
                                    ident.to_string(),
                                    (0, 0),
                                ))
                            }
                        }
                        fields.push((
                            get_ident(key),
                            get_type(step_inside_val(key, "type"), &mut errors),
                        ))
                    }
                    // check if already exists
                    for struct_ in &dictionary.structs {
                        if struct_.name == ident {
                            errors.push(ErrType::ConflictingNames(ident.to_string()))
                        }
                    }
                    dictionary.structs.push(Struct {
                        name: ident,
                        generics,
                        fields,
                        assign,
                        methods: Vec::new(),
                    });
                }
                "KWType" => {
                    let ident = get_ident(&node);
                    let kind = get_type(step_inside_val(&node, "type"), &mut errors);
                    let assign = get_assign(&node);
                    // check if already exists
                    for type_ in &dictionary.types {
                        if type_.name == ident {
                            errors.push(ErrType::ConflictingNames(ident.to_string()))
                        }
                    }
                    dictionary.types.push(Type {
                        name: ident,
                        kind,
                        assign,
                    });
                }
                "KWEnum" => {
                    let ident = get_ident(&node);
                    let assign = get_assign(&node);
                    let mut variants: Vec<(String, usize)> = Vec::new();
                    for key in step_inside_arr(node, "values") {
                        let ident = get_ident(&key);
                        for variant in &variants {
                            if *variant.0 == ident {
                                errors.push(ErrType::EnumVariantAssignedIdent(
                                    ident.to_string(),
                                    (0, 0),
                                ))
                            }
                        }
                        if let Tokens::Number(num, _) = step_inside_val(key, "default").name {
                            variants.push((ident, num as usize));
                        }else {
                            // use last + 1
                            if let Some(last) = variants.last() {
                                variants.push((ident, last.1 + 1));
                            }else {
                                variants.push((ident, 0));
                            }
                        }
                    }
                    // check if already exists
                    for enum_ in &dictionary.enums {
                        if enum_.name == ident {
                            errors.push(ErrType::ConflictingNames(ident.to_string()))
                        }
                    }
                    dictionary.enums.push(Enum {
                        name: ident,
                        variants,
                        assign,
                    });
                }
                "KWFun" => {
                    let fun = get_fun_siginifier(&node, &mut errors);
                    // check if already exists
                    for fun_ in &dictionary.functions {
                        if fun_.name == fun.name {
                            errors.push(ErrType::ConflictingNames(fun.name.to_string()))
                        }
                    }
                    dictionary.functions.push(fun);
                }
                "KWImpl" => {
                    // find struct and append methods
                    let ident = get_ident(&node);
                    for struct_ in &mut dictionary.structs {
                        if struct_.name == ident {
                            for method in step_inside_arr(node, "methods") {
                                let fun = get_fun_siginifier(&method, &mut errors);
                                // check if already exists
                                for fun_ in &struct_.methods {
                                    if fun_.name == fun.name {
                                        errors.push(ErrType::ConflictingNames(fun.name.to_string()))
                                    }
                                }
                                struct_.methods.push(fun);
                            }
                        }
                    }
                }
                "KWConst" => {
                    let ident = get_ident(&node);
                    let val = &step_inside_val(&node, "value");
                    let value = match &step_inside_val(val, "value").name {
                        Tokens::Number(n, c) => ConstValue::Number(*n, *c),
                        Tokens::String(text) => ConstValue::Text(text.to_string()),
                        Tokens::Text(bool) => match bool.as_str() {
                            "true" => ConstValue::Bool(true),
                            "false" => ConstValue::Bool(false),
                            _ => {
                                errors.push(ErrType::InvalidConstant(Tokens::Text(bool.to_string())));
                                ConstValue::Bool(false)
                            }
                        },
                        _ => panic!("hruzostrasna pohroma"),
                    };
                    // check if already exists
                    for const_ in &dictionary.consts {
                        if const_.name == ident {
                            errors.push(ErrType::ConflictingNames(ident.to_string()))
                        }
                    }
                    dictionary.consts.push(Const {
                        name: ident,
                        value,
                    });
                }
                _ => {}
            }
        } else {
            return Err("".to_owned());
        }
    }
    if errs.len() > 0 {
        for err in errs {
            println!("{:?}", err);
        }
        return Err("".to_owned());
    }
    //println!("\n\n{:#?}", dictionary);
    Ok(dictionary)
}

fn from_tree(node: &Node) -> Result<Dictionary, String> {
    let mut dict = Dictionary::new();
    let nodes = step_inside_arr(&node, "nodes");
    Ok(dict)
}

fn get_assign(node: &Node) -> usize {
    let node = step_inside_val(&node, "assign");
    if let Tokens::Number(num , _) = step_inside_val(node, "num").name {
        return num as usize;
    }
    //println!("node: {:?}", node);
    panic!("hruzostrasna pohroma");
}
fn get_fun_siginifier(node: &Node, errors: &mut Vec<ErrType>) -> Function {
    let mut args: Vec<(String, ShallowType, MemoryTypes)> = Vec::new();
    let mut return_type = if let Tokens::Text(txt) = &step_inside_val(node, "type").name {
        if txt == "type_specifier" {
            get_type(step_inside_val(step_inside_val(node, "type"), "type"), errors)
        }else {
            ShallowType::empty()
        }
    }else {
        ShallowType::empty()
    };
    let mut errorable = if let Tokens::Operator(Operators::Not) = step_inside_val(node, "errorable").name {
        true
    }else {
        false
    };
    Function {
        name: get_ident(node),
        args,
        return_type,
        errorable,
        assign: get_assign(node),
    }
}

fn get_mem_loc(node: &Node) -> MemoryTypes {
    let mem = if let Tokens::Text(txt) = &step_inside_val(&node, "mem").name {
        txt.to_string()
    } else {
        unreachable!("you somehow managed to break the compiler, gj");
    };
    let loc = if let Tokens::Text(txt) = &step_inside_val(&node, "loc").name {
        txt.to_string()
    } else {
        unreachable!("you somehow managed to break the compiler, gj");
    };
    match mem.to_lowercase().as_str() {
        "stack" => MemoryTypes::Stack(loc.parse::<usize>().unwrap()),
        "register" => {
            if let Some(reg) = Registers::from_str(&loc, &mut Vec::new()) {
                MemoryTypes::Register(reg)
            } else {
                MemoryTypes::Register(Registers::G1)
            }
        },
        _ => unreachable!("you somehow managed to break the compiler, gj"),
    }
}

#[derive(Debug)]
pub enum MemoryTypes {
    Stack(usize),
    Register(Registers),
}

#[derive(Debug)]
pub enum Registers {
    G1,
    G2,
    G3,
    G4,
    G5,
    G6,
    Ptr,
    Ret,
    CodePtr,
}

impl Registers {
    fn from_str(s: &str, errors: &mut Vec<ErrType>) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "g1" => Some(Registers::G1),
            "g2" => Some(Registers::G2),
            "g3" => Some(Registers::G3),
            "g4" => Some(Registers::G4),
            "g5" => Some(Registers::G5),
            "g6" => Some(Registers::G6),
            "ptr" => Some(Registers::Ptr),
            "ret" => Some(Registers::Ret),
            "cptr" => Some(Registers::CodePtr),
            _ => {
                errors.push(ErrType::InvalidRegister(s.to_string()));
                None
            },
        }
    }
}

#[derive(Debug)]
pub struct Dictionary {
    pub functions: Vec<Function>,
    pub structs: Vec<Struct>,
    pub enums: Vec<Enum>,
    pub types: Vec<Type>,
    pub consts: Vec<Const>,
    pub traits: Vec<Trait>,
}

impl Dictionary {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            structs: Vec::new(),
            enums: Vec::new(),
            types: Vec::new(),
            consts: Vec::new(),
            traits: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub args: Vec<(String, ShallowType, MemoryTypes)>,
    pub return_type: ShallowType,
    pub errorable: bool,
    pub assign: usize,
}

#[derive(Debug)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<(String, ShallowType)>,
    pub assign: usize,
    pub generics: Vec<GenericDecl>,
    pub methods: Vec<Function>,
}

#[derive(Debug)]
pub struct Enum {
    pub name: String,
    pub variants: Vec<(String, usize)>,
    pub assign: usize,
}

#[derive(Debug)]
pub struct Type {
    pub name: String,
    pub kind: ShallowType,
    pub assign: usize,
}

#[derive(Debug)]
pub struct Const {
    pub name: String,
    pub value: ConstValue,
}

#[derive(Debug)]
pub enum ConstValue {
    Number(f64, char),
    Text(String),
    Bool(bool),
}

#[derive(Debug)]
pub struct Trait {
    pub name: String,
    pub functions: Vec<Function>,
    pub assign: usize,
}