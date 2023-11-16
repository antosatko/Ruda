use std::collections::HashMap;

use crate::{
    ast_parser::ast_parser::{Head, HeadParam},
    intermediate::{self, AnalyzationError::ErrType},
    lexer::tokenizer::{self, Operators, Tokens},
    tree_walker::tree_walker::{generate_tree,  Line, Node},
};
use intermediate::dictionary::*;
use runtime::runtime_types::{GENERAL_REG1, GENERAL_REG2, GENERAL_REG3, MEMORY_REG1, MEMORY_REG2, ARGS_REG, POINTER_REG, RETURN_REG, CODE_PTR_REG};

pub fn load(
    string: &[u8],
    ast: &mut (HashMap<String, Head>, Vec<HeadParam>),
) -> Result<Dictionary, String> {
    let (tokens, lines, errs) = tokenizer::tokenize(string, true);
    let tree = match generate_tree(&tokens, ast, &lines) {
        Ok(tree) => tree,
        Err(err) => {
            return Err(format!("Failed to generate tree. {:?}", err));
        }
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
                                    field.1.line,
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
                            errors.push(ErrType::ConflictingNames(ident.to_string(), node.line))
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
                            errors.push(ErrType::ConflictingNames(ident.to_string(), node.line))
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
                                    key.line,
                                ))
                            }
                        }
                        if let Tokens::Number(num, _) = step_inside_val(key, "default").name {
                            variants.push((ident, num as usize));
                        } else {
                            // use last + 1
                            if let Some(last) = variants.last() {
                                variants.push((ident, last.1 + 1));
                            } else {
                                variants.push((ident, 0));
                            }
                        }
                    }
                    // check if already exists
                    for enum_ in &dictionary.enums {
                        if enum_.name == ident {
                            errors.push(ErrType::ConflictingNames(ident.to_string(), node.line))
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
                            errors.push(ErrType::ConflictingNames(fun.name.to_string(), node.line))
                        }
                    }
                    dictionary.functions.push(fun);
                }
                "KWOverload" => {
                    todo!("move to impl block");
                    let op = step_inside_val(&node, "op").name.clone();
                    let arg = {
                        let arg = step_inside_val(&node, "arg");
                        let identifier = get_ident(&arg);
                        let kind = get_type(&step_inside_val(&arg, "type"), &mut errors);
                        let mem_loc = get_mem_loc(&arg);
                        (identifier, kind, mem_loc)
                    };
                    let kind = step_inside_val(&step_inside_val(&node, "type"), "type");
                    let mem_loc = get_mem_loc(node);
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
                                        errors.push(ErrType::ConflictingNames(
                                            fun.name.to_string(),
                                            method.line,
                                        ))
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
                                errors.push(ErrType::InvalidConstant(
                                    Tokens::Text(bool.to_string()),
                                    val.line,
                                ));
                                ConstValue::Bool(false)
                            }
                        },
                        _ => panic!("hruzostrasna pohroma"),
                    };
                    // check if already exists
                    for const_ in &dictionary.consts {
                        if const_.name == ident {
                            errors.push(ErrType::ConflictingNames(ident.to_string(), node.line))
                        }
                    }
                    dictionary.consts.push(Const { name: ident, value });
                }
                "KWUserdata" => {
                    let ident = get_ident(&node);
                    let assign = get_assign(&node);
                    let generics = {
                        let gen = &step_inside_val(&node, "generics");
                        match &gen.name {
                            Tokens::Text(txt) => match txt.as_str() {
                                "'none" => Vec::new(),
                                _ => get_generics_decl(gen, &mut errors),
                            },
                            _ => unreachable!("you somehow managed to break the compiler, gj"),
                        }  
                    };
                    // check if already exists
                    for ud in &dictionary.user_data {
                        if ud.name == ident {
                            errors.push(ErrType::ConflictingNames(ident.to_string(), node.line))
                        }
                    }
                    dictionary.user_data.push(UserData {
                        name: ident,
                        assign,
                        generics,
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

fn get_assign(node: &Node) -> usize {
    let node = step_inside_val(&node, "assign");
    if let Tokens::Number(num, _) = step_inside_val(node, "num").name {
        return num as usize;
    }
    //println!("node: {:?}", node);
    panic!("hruzostrasna pohroma");
}
fn get_fun_siginifier(node: &Node, errors: &mut Vec<ErrType>) -> Function {
    let mut args: Vec<(String, ShallowType, MemoryTypes)> = Vec::new();
    for arg in step_inside_arr(node, "arguments") {
        if let Tokens::Text(txt) = &arg.name {
            if txt == "self_arg" {
                let ident = "self".to_string();
                let arg_type = ShallowType::empty();
                let mem_loc = get_mem_loc(&arg);
                args.push((ident, arg_type, mem_loc));
                continue;
            }
        }
        let ident = get_ident(&arg);
        let mem_loc = get_mem_loc(&arg);
        let arg_type = get_type(step_inside_val(&arg, "type"), errors);
        args.push((ident, arg_type, mem_loc));
    }
    let return_type = if let Tokens::Text(txt) = &step_inside_val(node, "type").name {
        if txt == "type_specifier" {
            get_type(
                step_inside_val(step_inside_val(node, "type"), "type"),
                errors,
            )
        } else {
            ShallowType::empty()
        }
    } else {
        ShallowType::empty()
    };
    let errorable =
        if let Tokens::Operator(Operators::Not) = step_inside_val(node, "errorable").name {
            true
        } else {
            false
        };
    // TODO: get args
    Function {
        name: get_ident(node),
        args,
        return_type,
        errorable,
        assign: get_assign(node),
    }
}

fn get_mem_loc(node: &Node) -> MemoryTypes {
    let node = step_inside_val(&node, "mem");
    let mem =
        if let Tokens::Text(txt) = &step_inside_val(&step_inside_val(&node, "mem"), "mem").name {
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
        "reg" => {
            if let Some(reg) = Registers::from_str(&loc, &mut Vec::new(), node.line) {
                MemoryTypes::Register(reg)
            } else {
                MemoryTypes::Register(Registers::G1)
            }
        }
        _ => unreachable!("you somehow managed to break the compiler, gj"),
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MemoryTypes {
    Stack(usize),
    Register(Registers),
}

impl MemoryTypes {
    pub fn add(&self, n: i64) -> Self {
        match self {
            MemoryTypes::Stack(num) => MemoryTypes::Stack(((*num as i64) + n) as usize),
            MemoryTypes::Register(reg) => MemoryTypes::Register(*reg),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Registers {
    G1,
    G2,
    G3,
    G4,
    G5,
    Args,
    Ptr,
    Ret,
    CodePtr,
}

impl Registers {
    fn from_str(s: &str, errors: &mut Vec<ErrType>, line: Line) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "g1" => Some(Registers::G1),
            "g2" => Some(Registers::G2),
            "g3" => Some(Registers::G3),
            "g4" => Some(Registers::G4),
            "g5" => Some(Registers::G5),
            "args" => Some(Registers::Args),
            "ptr" => Some(Registers::Ptr),
            "ret" => Some(Registers::Ret),
            "cptr" => Some(Registers::CodePtr),
            _ => {
                errors.push(ErrType::InvalidRegister(s.to_string(), line));
                None
            }
        }
    }
    pub fn to_num(&self) -> usize {
        match self {
            Registers::G1 => GENERAL_REG1,
            Registers::G2 => GENERAL_REG2,
            Registers::G3 => GENERAL_REG3,
            Registers::G4 => MEMORY_REG1,
            Registers::G5 => MEMORY_REG2,
            Registers::Args => ARGS_REG,
            Registers::Ptr => POINTER_REG,
            Registers::Ret => RETURN_REG,
            Registers::CodePtr => CODE_PTR_REG,
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
    pub user_data: Vec<UserData>,
    pub id: usize,
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
            user_data: Vec::new(),
            id: 0,
        }
    }
}

#[derive(Debug)]
pub struct UserData {
    pub name: String,
    pub assign: usize,
    pub generics: Vec<GenericDecl>,
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
