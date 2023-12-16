use intermediate::dictionary::ImportKinds;
use std::collections::HashMap;
use std::vec;

use runtime::runtime_types::{
    self, Instructions, Memory, Stack, Types, ARGS_REG, CODE_PTR_REG, GENERAL_REG1, GENERAL_REG2,
    GENERAL_REG3, MEMORY_REG1, POINTER_REG, RETURN_REG,
};

use crate::codeblock_parser::Nodes;
use crate::expression_parser::{self, ArrayRule, FunctionCall, Ref, Root, ValueType};
use crate::intermediate::dictionary::{
    self, Arg, ConstValue, Function, ShTypeBuilder, ShallowType, TypeComparison,
};
use crate::intermediate::{Kind, TypeBody};
use crate::lexer::tokenizer::{self, Operators};
use crate::tree_walker::tree_walker::Line;
use crate::{intermediate, prep_objects::Context};

use crate::libloader::{self, MemoryTypes};

const CORE_LIB: usize = 4;

pub fn gen(
    objects: &mut Context,
    main: &str,
) -> Result<runtime::runtime_types::Context, CodegenError> {
    let mut vm_context = runtime::runtime_types::Context::new(Vec::new());
    // Initialize some common constants for faster lookup
    let consts: [ConstValue; 9] = [
        ConstValue::Null,
        ConstValue::Bool(true),
        ConstValue::Bool(false),
        ConstValue::Usize(0),
        ConstValue::Int(0),
        ConstValue::Int(1),
        ConstValue::Int(-1),
        ConstValue::Usize(1),
        ConstValue::Usize(2),
    ];
    vm_context
        .memory
        .stack
        .data
        .extend(consts.iter().map(|c| c.to_runtime()));
    let main_path = InnerPath::main();
    let fun_locs = gen_all_fun_ids(objects)?;
    gen_all_funs(objects, &mut vm_context)?;
    fix_fun_calls(objects, &mut vm_context, &fun_locs)?;
    call_main(main_path.get(objects)?, &mut vm_context)?;
    Ok(vm_context)
}

fn call_main(main: &Function, context: &mut runtime_types::Context) -> Result<(), CodegenError> {
    use Instructions::*;
    context.code.entry_point = context.code.data.len() + 1;
    let consts_len = context.memory.stack.data.len();
    context
        .code
        .data
        .extend(&[End, ReserveStack(consts_len, 0), Goto(main.location)]);
    // swap all returns in main with end
    for i in main.location..main.instrs_end {
        match context.code.data[i] {
            Return => {
                context.code.data[i] = End;
            }
            _ => {}
        }
    }

    Ok(())
}

fn fix_fun_calls(
    objects: &Context,
    context: &mut runtime_types::Context,
    fun_ids: &Vec<InnerPath>,
) -> Result<(), CodegenError> {
    for instr in context.code.data.iter_mut() {
        match instr {
            Instructions::Jump(id) => {
                let fun = fun_ids[*id].get(objects)?;
                *id = fun.location;
            }
            _ => {}
        }
    }
    Ok(())
}

/// Gives all functions unique ids before generating them,
/// then if a function wants to call another function,
/// it can just use the id, which will be replaced with
/// the actual location of the function later
fn gen_all_fun_ids(objects: &mut Context) -> Result<Vec<InnerPath>, CodegenError> {
    let keys = objects.0.keys().map(|s| s.clone()).collect::<Vec<_>>();
    let mut paths = Vec::new();
    let mut id = 0;
    // do main first
    let main = InnerPath::main();
    paths.push(main.clone());
    let main = main.get_mut(objects).unwrap();
    main.id = id;
    id += 1;

    for file in keys {
        for fun in 0..objects.0.get(&file).unwrap().functions.len() {
            let fun = InnerPath {
                file: file.clone(),
                block: None,
                ident: objects.0.get(&file).unwrap().functions[fun]
                    .identifier
                    .clone()
                    .unwrap(),
                kind: ImportKinds::Rd,
            };
            // skip main
            if fun.ident == "main" {
                continue;
            }
            paths.push(fun.clone());
            let fun = fun.get_mut(objects).unwrap();
            fun.id = id;
            id += 1;
        }
        for structt in 0..objects.0.get(&file).unwrap().structs.len() {
            let path = InnerPath {
                file: file.clone(),
                block: Some(
                    objects.0.get(&file).unwrap().structs[structt]
                        .identifier
                        .clone(),
                ),
                ident: "".to_string(),
                kind: ImportKinds::Rd,
            };
            for fun in 0..objects.0.get(&file).unwrap().structs[structt]
                .functions
                .len()
            {
                let mut fun_path = path.clone();
                fun_path.ident = objects.0.get(&file).unwrap().structs[structt].functions[fun]
                    .identifier
                    .clone()
                    .unwrap();
                paths.push(fun_path.clone());
                let fun = match fun_path.get_mut(objects) {
                    Ok(fun) => fun,
                    Err(err) => {
                        return Err(err);
                    }
                };
                fun.id = id;
                id += 1;
            }
        }
    }

    Ok(paths)
}

fn gen_all_funs(
    objects: &mut Context,
    context: &mut runtime_types::Context,
) -> Result<(), CodegenError> {
    let keys = objects.0.keys().map(|s| s.clone()).collect::<Vec<_>>();
    for file in keys {
        for fun in 0..objects.0.get(&file).unwrap().functions.len() {
            let fun = InnerPath {
                file: file.clone(),
                block: None,
                ident: objects.0.get(&file).unwrap().functions[fun]
                    .identifier
                    .clone()
                    .unwrap(),
                kind: ImportKinds::Rd,
            };
            gen_fun(objects, &fun, context, false)?;
        }
        for structt in 0..objects.0.get(&file).unwrap().structs.len() {
            let path = InnerPath {
                file: file.clone(),
                block: Some(
                    objects.0.get(&file).unwrap().structs[structt]
                        .identifier
                        .clone(),
                ),
                ident: "".to_string(),
                kind: ImportKinds::Rd,
            };
            for fun in 0..objects.0.get(&file).unwrap().structs[structt]
                .functions
                .len()
            {
                let mut fun_path = path.clone();
                let mut is_constructor = false;
                match objects.0.get(&file).unwrap().structs[structt].functions[fun].identifier {
                    Some(ref ident) => {
                        fun_path.ident = ident.clone();
                        // if the name of the function is the same as the struct
                        // then it's a constructor
                        if ident == &path.block.clone().unwrap() {
                            is_constructor = true;
                        }
                    }
                    None => {
                        fun_path.ident = path.block.clone().unwrap();
                    }
                }
                gen_fun(objects, &fun_path, context, is_constructor)?;
            }
        }
    }
    Ok(())
}

#[derive(Debug)]
pub enum CodegenError {
    CannotInitializeConstant,
    FunctionNotFound(InnerPath),
    ExpectedBool(Line),
    /// (depth, line)
    DerefereString(usize, Line),
    /// (depth, line)
    ReferenceString(usize, Line),
    /// (expected, got, comparison, line)
    VariableTypeMismatch(Kind, Kind, TypeComparison, Line),
    /// (line)
    NotInitializedNoType(Line),
    /// (line)
    TypeNotNullable(Line),
    /// (ident, this_line, other_line)
    VariableAlreadyDeclared(String, Line, Line),
    VariableNotFound(String, Line),
    /// (line)
    CannotIndexFile(Line),
    CannotIndexFunction(Line),
    CanCallOnlyFunctions(Line),
    // (expected, got, comparison, line)
    ArgTypeMismatch(Arg, Kind, TypeComparison, Line),
    CannotAttachMethodsToFunctions(Line),
    ImportIsNotAValidValue(Line),
    IncorrectArgs(Line),
    ExressionNotHandledProperly(Line),
    InvalidOperator(Kind, Kind, Operators, Line),
    // (to, from, line)
    CouldNotCastTo(Kind, Kind, Line),
    // (line)
    ExpectedNumber(Line),
    // (kind, unary, line)s
    UnaryNotApplicable(Kind, Operators, Line),
    CoudNotCastAnArrayToANonArray(Kind, Line),
    FunctionDoesNotReturn(Line),
    /// (expected, got, line)
    IncorrectNumberOfArgs(usize, usize, Line),
    ImportNotFound(String, Line),
    KindNotFound(String, Line),
    CannotRefDerefNumLiteral(Line),
    CannotIndexNonArray(Position, Line),
    FieldNotInStruct(String, Line),
    CannotDereference(usize, Kind, Line),
    CannotReference(usize, Kind, Line),
    CannotGetKind(Position),
    CannotCallType(Kind, Line),
    CannotCastNull(Kind, Line),
    SwitchWithoutCases(Line),
}

pub fn stringify(
    context: &runtime::runtime_types::Context,
    shlibs: &Vec<stringify::ShLib>,
) -> String {
    use stringify::stringify;
    stringify(context, Some(&shlibs))
}

/// Returns whether the function was successfully generated
fn gen_fun<'a>(
    objects: &'a mut Context,
    fun: &'a InnerPath,
    context: &'a mut runtime_types::Context,
    is_constructor: bool,
) -> Result<bool, CodegenError> {
    let mut code = Code { code: Vec::new() };
    let mut args_scope_len = 0;
    let mut scopes = vec![ScopeCached {
        variables: HashMap::new(),
    }];
    let this_fun = fun.get(objects)?;
    let takes_self = this_fun.takes_self;
    if takes_self {
        let structt = objects
            .0
            .get(&fun.file)
            .unwrap()
            .structs
            .iter()
            .find(|struc| struc.identifier == fun.block.clone().unwrap())
            .clone()
            .unwrap();
        let struct_kind = Kind::from_struct(
            structt.identifier.clone(),
            fun.file.clone(),
            structt.line.clone(),
        );
        let pos = create_var_pos(&scopes);
        scopes[0].variables.insert(
            "self".to_string(),
            Variable {
                kind: Some(struct_kind.clone()),
                pos,
                value: None,
                line: this_fun.line.clone(),
            },
        );
        args_scope_len += 1;
    }
    for arg in this_fun.args.iter() {
        let pos = create_var_pos(&scopes);
        scopes[0].variables.insert(
            arg.identifier.clone(),
            Variable {
                kind: Some(arg.kind.clone()),
                pos,
                value: None,
                line: arg.line.clone(),
            },
        );
        args_scope_len += 1;
    }
    if is_constructor {
        scopes.push(ScopeCached {
            variables: HashMap::new(),
        });
        let structt = objects
            .0
            .get(&fun.file)
            .unwrap()
            .structs
            .iter()
            .find(|struc| struc.identifier == fun.block.clone().unwrap())
            .clone()
            .unwrap();
        let pos = create_var_pos(&scopes);
        scopes[1].variables.insert(
            "self".to_string(),
            Variable {
                kind: Some(Kind::from_struct(
                    structt.identifier.clone(),
                    fun.file.clone(),
                    structt.line.clone(),
                )),
                pos,
                value: None,
                line: this_fun.line.clone(),
            },
        );
        args_scope_len += 1;
        code.extend(&[Instructions::AllocateStatic(structt.fields.len() + 1)]);
        code.write(POINTER_REG, &pos);
    }
    let scope_len = {
        let (scope_len, terminator) = get_scope(
            objects,
            &this_fun.code.clone(),
            context,
            &mut scopes,
            &mut code,
            fun,
        )?;
        let this_fun = fun.get(objects)?;
        if terminator != ScopeTerminator::Return
            && this_fun.return_type.is_some()
            && !is_constructor
        {
            Err(CodegenError::FunctionDoesNotReturn(this_fun.line.clone()))?;
        } else if this_fun.return_type.is_none() && !is_constructor {
            let null = 0;
            code.extend(&[Instructions::ReadConst(null, RETURN_REG)]);
            code.push(Instructions::Return);
        }
        scope_len + args_scope_len
    };
    if is_constructor {
        code.read(&scopes[1].variables.get("self").unwrap().pos, RETURN_REG);
    }
    flip_stack_access(scope_len, &mut code);
    code.push(Instructions::Return);
    let mut args_code = Code {
        code: vec![Instructions::ReserveStack(scope_len, 0)],
    };
    for (idx, _) in scopes[0].variables.iter().enumerate() {
        args_code.extend(&[
            Instructions::Move(ARGS_REG, POINTER_REG),
            Instructions::IndexStatic(idx),
            Instructions::ReadPtr(GENERAL_REG1),
            Instructions::Write(scope_len - idx, GENERAL_REG1),
        ]);
    }
    let mut temp = Vec::new();
    merge_code(&mut temp, &args_code.code, scope_len);
    merge_code(&mut temp, &code.code, scope_len);
    let pos = merge_code(&mut context.code.data, &temp, scope_len);
    let this_fun = fun.get_mut(objects)?;
    this_fun.location = pos.0;
    this_fun.instrs_end = pos.1;
    this_fun.stack_size = Some(scope_len);
    Ok(true)
}

/// evaluates expression at runtime and puts result in reg1
fn expression(
    objects: &mut Context,
    expr: &expression_parser::ValueType,
    scopes: &mut Vec<ScopeCached>,
    code: &mut Code,
    context: &mut runtime_types::Context,
    fun: &InnerPath,
    scope_len: &mut usize,
    expected_type: Option<Kind>,
    line: Line,
) -> Result<Kind, CodegenError> {
    use Instructions::*;
    let mut return_kind = Kind::void();
    let expected_type = match expected_type {
        Some(kind) => Some(correct_kind(objects, &kind, fun, &line)?),
        None => None,
    };
    match expr {
        ValueType::AnonymousFunction(_) => todo!(),
        ValueType::Expression(expr) => {
            let left = match expr.left.as_ref() {
                Some(left) => left,
                None => Err(CodegenError::ExressionNotHandledProperly(expr.line.clone()))?,
            };
            let right = match expr.right.as_ref() {
                Some(right) => right,
                None => Err(CodegenError::ExressionNotHandledProperly(expr.line.clone()))?,
            };
            let left_kind = expression(
                objects, left, scopes, code, context, &fun, scope_len, None, line,
            )?;
            *scope_len += 1;
            let var = create_var_pos(scopes);
            let len = scopes.len();
            scopes[len - 1].variables.insert(
                scope_len.to_string(),
                Variable {
                    kind: Some(left_kind.clone()),
                    pos: var.clone(),
                    value: None,
                    line: expr.line.clone(),
                },
            );
            code.write(GENERAL_REG1, &var);
            let right_kind = expression(
                objects,
                right,
                scopes,
                code,
                context,
                &fun,
                scope_len,
                Some(left_kind.clone()),
                line,
            )?;
            code.read(&var, GENERAL_REG2);
            code.push(Swap(GENERAL_REG1, GENERAL_REG2));
            let op = match &expr.operator {
                Some(op) => op,
                None => Err(CodegenError::ExressionNotHandledProperly(expr.line.clone()))?,
            };
            let kind = match native_operand(
                objects,
                &op,
                &left_kind,
                &right_kind,
                code,
                context,
                &fun,
                &expr.line,
            ) {
                Some(kind) => kind,
                None => {
                    todo!("non native operand");
                }
            };
            return_kind = kind;
        }
        ValueType::Operator(_, line) => unreachable!(
            "operator not handled properly by the compiler at {line}, please report this bug"
        ),
        ValueType::Value(value) => {
            return_kind = gen_value(
                objects,
                value,
                context,
                scopes,
                code,
                fun,
                scope_len,
                ExpectedValueType::Value,
                expected_type.clone(),
            )?
            .get_kind()?;
            //}
            for un in value.unary.iter() {
                return_kind = native_unary_operand(
                    objects,
                    &Some(*un),
                    &return_kind,
                    code,
                    context,
                    &fun,
                    &line,
                    GENERAL_REG1,
                )?;
            }
        }
        ValueType::Blank => {}
    }
    if let Some(expected_type) = expected_type {
        let cmp = expected_type.cmp(&return_kind);
        match cast(
            objects,
            &mut return_kind,
            &expected_type,
            code,
            context,
            &fun,
            &line,
            GENERAL_REG1,
        ) {
            Some(_) => {
                return_kind = expected_type;
            }
            None => {
                return Err(CodegenError::CouldNotCastTo(
                    expected_type,
                    return_kind,
                    line.clone(),
                ));
            }
        }
    }
    Ok(return_kind)
}

fn find_var<'a>(scopes: &'a Vec<ScopeCached>, ident: &'a str) -> Option<&'a Variable> {
    for scope in scopes.iter().rev() {
        if let Some(var) = scope.variables.get(ident) {
            return Some(var);
        }
    }
    None
}

fn find_import<'a>(
    objects: &'a Context,
    ident: &'a str,
    file_name: &'a str,
) -> Option<(&'a str, intermediate::dictionary::ImportKinds)> {
    match objects.0.get(file_name) {
        Some(dictionary) => {
            for import in dictionary.imports.iter() {
                if import.alias == ident {
                    return Some((&import.path, import.kind.clone()));
                }
            }
        }
        None => None?,
    };
    None
}

fn find_fun<'a>(objects: &'a Context, ident: &'a str, file_name: &'a str) -> Option<FunctionKind> {
    match objects.0.get(file_name) {
        Some(dictionary) => {
            for fun in dictionary.functions.iter() {
                if fun.identifier.clone().unwrap() == ident {
                    return Some(FunctionKind::Fun(InnerPath {
                        file: file_name.to_string(),
                        block: None,
                        ident: ident.to_string(),
                        kind: ImportKinds::Rd,
                    }));
                }
            }
        }
        None => (),
    };
    match objects.1.get(file_name) {
        Some(dictionary) => {
            for fun in dictionary.functions.iter() {
                if fun.name.clone() == ident {
                    return Some(FunctionKind::Binary(InnerPath {
                        file: file_name.to_string(),
                        block: None,
                        ident: ident.to_string(),
                        kind: ImportKinds::Dll,
                    }));
                }
            }
        }
        None => None?,
    };
    None
}

fn find_struct<'a>(
    objects: &'a Context,
    ident: &'a str,
    file_name: &'a str,
) -> Option<(&'a str, &'a dictionary::Struct)> {
    match objects.0.get(file_name) {
        Some(dictionary) => {
            for struc in dictionary.structs.iter() {
                if struc.identifier == ident {
                    return Some((file_name, struc));
                }
            }
        }
        None => (),
    };
    None
}

fn find_userdata<'a>(
    objects: &'a Context,
    ident: &'a str,
    file_name: &'a str,
) -> Option<(&'a str, &'a libloader::UserData)> {
    match objects.1.get(file_name) {
        Some(dictionary) => {
            for userdata in dictionary.user_data.iter() {
                if userdata.name == ident {
                    return Some((file_name, userdata));
                }
            }
        }
        None => (),
    };
    None
}

#[derive(Debug, Clone)]
enum FunctionKind {
    Fun(InnerPath),
    Binary(InnerPath),
    Dynamic(Kind),
}

fn gen_value(
    objects: &mut Context,
    value: &expression_parser::Variable,
    context: &mut runtime_types::Context,
    scopes: &mut Vec<ScopeCached>,
    code: &mut Code,
    fun: &InnerPath,
    scope_len: &mut usize,
    expected_type: ExpectedValueType,
    expected_kind: Option<Kind>,
) -> Result<Position, CodegenError> {
    use Instructions::*;
    let root = identify_root(
        objects,
        &value.root.0,
        Some(scopes),
        &fun.file,
        &value.root.1,
        context,
        code,
        scope_len,
        expected_kind,
        fun,
    )?;
    let pos = traverse_tail(
        objects,
        &mut value.tail.iter(),
        context,
        scopes,
        code,
        fun,
        root,
        scope_len,
    )?;
    let kind = match &pos {
        Position::CompoundField(path, field, kind) => {
            let ident = match field {
                CompoundField::Field(ident) => ident,
                _ => todo!(),
            };
            let structt = find_struct(objects, &path.ident, kind.file.as_ref().unwrap())
                .unwrap()
                .1;
            let kind = structt
                .fields
                .iter()
                .find(|f| f.0 == field.get_ident())
                .unwrap()
                .1
                .clone();
            match value.refs {
                expression_parser::Ref::Dereferencing(depth) => {
                    if depth > kind.get_refs() {
                        Err(CodegenError::CannotDereference(
                            depth,
                            kind.clone(),
                            value.root.1.clone(),
                        ))?;
                    }
                    let start = match expected_type {
                        ExpectedValueType::Pointer => 1,
                        ExpectedValueType::Value => 0,
                    };
                    for _ in start..depth + 1 {
                        code.extend(&[ReadPtr(POINTER_REG)]);
                    }
                    code.push(Move(POINTER_REG, GENERAL_REG1));
                    let mut kind = kind.clone();
                    *kind.refs_mut() -= depth;
                    Position::Pointer(kind)
                }
                expression_parser::Ref::Reference(depth) => {
                    if depth > 1 {
                        Err(CodegenError::CannotReference(
                            depth,
                            kind.clone(),
                            value.root.1.clone(),
                        ))?;
                    }
                    todo!();
                    pos
                }
                expression_parser::Ref::None => match expected_type {
                    ExpectedValueType::Pointer => Position::Pointer(kind),
                    ExpectedValueType::Value => {
                        code.extend(&[ReadPtr(GENERAL_REG1)]);
                        Position::Value(kind)
                    }
                },
            }
        }
        Position::Import(_) => Err(CodegenError::ImportIsNotAValidValue(value.root.1.clone()))?,
        Position::BinImport(_) => Err(CodegenError::ImportIsNotAValidValue(value.root.1.clone()))?,
        Position::Variable(var, kind) => {
            let var = match find_var(&scopes, &var) {
                Some(var) => var,
                None => Err(CodegenError::VariableNotFound(
                    var.clone(),
                    value.root.1.clone(),
                ))?,
            };
            let pos_cloned = var.pos.clone();
            match value.refs {
                expression_parser::Ref::Dereferencing(depth) => {
                    if depth > kind.get_refs() {
                        Err(CodegenError::CannotDereference(
                            depth,
                            kind.clone(),
                            value.root.1.clone(),
                        ))?;
                    }
                    code.read(&pos_cloned, POINTER_REG);
                    let start = match expected_type {
                        ExpectedValueType::Pointer => 1,
                        ExpectedValueType::Value => 0,
                    };
                    for _ in start..depth {
                        code.extend(&[ReadPtr(POINTER_REG)]);
                    }
                    code.push(Move(POINTER_REG, GENERAL_REG1));
                    let mut kind = var.kind.as_ref().unwrap().clone();
                    *kind.refs_mut() -= depth;
                    Position::Pointer(kind)
                }
                expression_parser::Ref::Reference(depth) => {
                    if depth > 1 {
                        Err(CodegenError::CannotReference(
                            depth,
                            kind.clone(),
                            value.root.1.clone(),
                        ))?;
                    }
                    code.ptr(&pos_cloned, GENERAL_REG1);
                    let mut kind = var.kind.as_ref().unwrap().clone();
                    *kind.refs_mut() += 1;
                    Position::Pointer(kind)
                }
                expression_parser::Ref::None => {
                    code.read(&pos_cloned, GENERAL_REG1);
                    pos
                }
            }
        }
        Position::Pointer(kind) => match value.refs {
            expression_parser::Ref::Dereferencing(depth) => {
                if depth > kind.get_refs() {
                    Err(CodegenError::CannotDereference(
                        depth,
                        kind.clone(),
                        value.root.1.clone(),
                    ))?;
                }
                code.extend(&[ReadPtr(GENERAL_REG1)]);
                let start = match expected_type {
                    ExpectedValueType::Pointer => 1,
                    ExpectedValueType::Value => 0,
                };
                for _ in start..depth {
                    code.extend(&[ReadPtr(GENERAL_REG1)]);
                }
                code.push(Move(GENERAL_REG1, GENERAL_REG1));
                let mut kind = kind.clone();
                *kind.refs_mut() -= depth;
                Position::Pointer(kind)
            }
            expression_parser::Ref::Reference(depth) => {
                if depth > 1 {
                    Err(CodegenError::CannotReference(
                        depth,
                        kind.clone(),
                        value.root.1.clone(),
                    ))?;
                }
                todo!();
                pos
            }
            expression_parser::Ref::None => match expected_type {
                ExpectedValueType::Pointer => pos,
                ExpectedValueType::Value => {
                    code.extend(&[ReadPtr(GENERAL_REG1)]);
                    Position::Value(kind.clone())
                }
            },
        },
        _ => pos,
    };
    Ok(kind)
}

fn identify_root(
    objects: &mut Context,
    ident: &Root,
    scopes: Option<&mut Vec<ScopeCached>>,
    file: &str,
    line: &Line,
    context: &mut runtime_types::Context,
    code: &mut Code,
    scope_len: &mut usize,
    expected_type: Option<Kind>,
    fun: &InnerPath,
) -> Result<Position, CodegenError> {
    use Instructions::*;
    match ident {
        Root::Identifier(ident) => {
            match ident.as_str() {
                "true" => {
                    code.push(ReadConst(1, GENERAL_REG1));
                    return Ok(Position::Value(Kind {
                        body: TypeBody::Type {
                            refs: 0,
                            main: vec!["bool".to_string()],
                            generics: vec![],
                            nullable: false,
                            kind: dictionary::KindType::Primitive,
                        },
                        line: line.clone(),
                        file: Some(file.to_string()),
                    }));
                }
                "false" => {
                    code.push(ReadConst(2, GENERAL_REG1));
                    return Ok(Position::Value(Kind {
                        body: TypeBody::Type {
                            refs: 0,
                            main: vec!["bool".to_string()],
                            generics: vec![],
                            nullable: false,
                            kind: dictionary::KindType::Primitive,
                        },
                        line: line.clone(),
                        file: Some(file.to_string()),
                    }));
                }
                "null" => {
                    code.push(ReadConst(0, GENERAL_REG1));
                    return Ok(Position::Value(Kind {
                        body: TypeBody::Type {
                            refs: 0,
                            main: vec!["null".to_string()],
                            generics: vec![],
                            nullable: false,
                            kind: dictionary::KindType::Primitive,
                        },
                        line: line.clone(),
                        file: Some(file.to_string()),
                    }));
                }
                _ => (),
            }
            if let Some(scopes) = scopes {
                if let Some(var) = find_var(scopes, &ident) {
                    return Ok(Position::Variable(
                        ident.to_string(),
                        var.kind.clone().unwrap(),
                    ));
                }
            }
            if let Some((fname, kind)) = find_import(objects, &ident, &file) {
                match kind {
                    dictionary::ImportKinds::Dll => {
                        return Ok(Position::BinImport(fname.to_string()));
                    }
                    dictionary::ImportKinds::Rd => {
                        return Ok(Position::Import(fname.to_string()));
                    }
                }
            }
            if let Some(fun) = find_fun(objects, &ident, &file) {
                let kind = match &fun {
                    FunctionKind::Fun(fun) => {
                        let fun = fun.get(objects)?;
                        fun.return_type.clone().unwrap_or(Kind {
                            body: TypeBody::Type {
                                refs: 0,
                                main: vec!["null".to_string()],
                                generics: vec![],
                                nullable: false,
                                kind: dictionary::KindType::Primitive,
                            },
                            line: line.clone(),
                            file: Some(file.to_string()),
                        })
                    }
                    FunctionKind::Binary(fun) => {
                        let fun = fun.get_bin(objects)?;
                        fun.return_type.clone()
                    }
                    FunctionKind::Dynamic(kind) => kind.clone(),
                };
                return Ok(Position::Function(fun, kind));
            }
            if let Some((fname, struc)) = find_struct(objects, &ident, &file) {
                return Ok(Position::Compound(Kind::from_struct(
                    struc.identifier.to_string(),
                    fname.to_string(),
                    struc.line,
                )));
            }
            if let Some((fname, userdata)) = find_userdata(objects, &ident, &file) {
                return Ok(Position::Compound(Kind::from_userdata(
                    userdata.name.to_string(),
                    fname.to_string(),
                    userdata.line,
                )));
            }
            Err(CodegenError::VariableNotFound(
                ident.to_string(),
                line.clone(),
            ))?
        }
        Root::Literal(lit) => {
            match lit {
                expression_parser::Literals::Number(n) => {
                    let const_num = match n.into_const_number(line.clone()) {
                        Some(num) => num,
                        None => {
                            unreachable!("number not handled properly by the compiler, please report this bug");
                        }
                    };
                    let pos = new_const(context, &const_num.0)?;
                    code.push(ReadConst(pos, GENERAL_REG1));
                    return Ok(Position::Value(const_num.1));
                }
                expression_parser::Literals::Array(rule) => match rule {
                    ArrayRule::Explicit(arr) => {
                        let mut kind = match &expected_type {
                            Some(kind) => match &kind.body {
                                TypeBody::Array {
                                    type_,
                                    size,
                                    refs,
                                    nullable,
                                } => Some(correct_kind(objects, &type_, fun, line)?),
                                _ => None
                            },
                            None => None,
                        };
                        if let Some(scopes) = scopes {
                            code.extend(&[AllocateStatic(arr.len())]);
                            *scope_len += 1;
                            let obj = create_var_pos(scopes);
                            let scopes_len = scopes.len();
                            scopes[scopes_len - 1].variables.insert(
                                scope_len.to_string(),
                                Variable {
                                    kind: Some(Kind::void()),
                                    pos: obj.clone(),
                                    value: None,
                                    line: line.clone(),
                                },
                            );
                            code.write(POINTER_REG, &obj);
                            for (idx, value) in arr.iter().enumerate() {
                                let temp_kind = Some(expression(
                                    objects,
                                    value,
                                    scopes,
                                    code,
                                    context,
                                    &fun,
                                    scope_len,
                                    kind.clone(),
                                    *line,
                                )?);
                                match &kind {
                                    Some(kind) => {
                                        match cast(
                                            objects,
                                            &mut kind.clone(),
                                            &temp_kind.clone().unwrap(),
                                            code,
                                            context,
                                            &fun,
                                            &line,
                                            GENERAL_REG1,
                                        ) {
                                            Some(_) => {
                                                // do nothing
                                            }
                                            None => {
                                                Err(CodegenError::CouldNotCastTo(
                                                    kind.clone(),
                                                    temp_kind.clone().unwrap(),
                                                    line.clone(),
                                                ))?;
                                            }
                                        }
                                    }
                                    None => {
                                        kind = temp_kind.clone();
                                    }
                                }
                                code.read(&obj, POINTER_REG);
                                code.extend(&[IndexStatic(idx), WritePtr(GENERAL_REG1)]);
                            }
                            code.read(&obj, GENERAL_REG1);
                            let mut return_kind = Kind {
                                body: TypeBody::Array {
                                    type_: Box::new(kind.clone().unwrap()),
                                    size: arr.len(),
                                    refs: 0,
                                    nullable: false,
                                },
                                line: line.clone(),
                                file: Some(file.to_string()),
                            };
                            return Ok(Position::Value(return_kind));
                        } else {
                            unreachable!("array not handled properly by the compiler, please report this bug")
                        }
                    }
                    ArrayRule::Fill { value, size } => {
                        let expected = match expected_type {
                            Some(ref kind) => match &kind.body {
                                TypeBody::Array {
                                    type_,
                                    size,
                                    refs,
                                    nullable,
                                } => Some(correct_kind(objects, &type_, fun, line)?),
                                _ => None
                            },
                            None => None,
                        };
                        if let Some(scopes) = scopes {
                            let value = expression(
                                objects,
                                value,
                                scopes,
                                code,
                                context,
                                &fun,
                                scope_len,
                                expected.clone(),
                                *line,
                            )?;
                            *scope_len += 1;
                            let value_var = create_var_pos(scopes);
                            let scopes_len = scopes.len();
                            scopes[scopes_len - 1].variables.insert(
                                scope_len.to_string(),
                                Variable {
                                    kind: Some(value.clone()),
                                    pos: value_var.clone(),
                                    value: None,
                                    line: line.clone(),
                                },
                            );
                            code.write(GENERAL_REG1, &value_var);
                            expression(
                                objects,
                                size,
                                scopes,
                                code,
                                context,
                                &fun,
                                scope_len,
                                Some(Kind {
                                    body: TypeBody::Type {
                                        refs: 0,
                                        main: vec!["uint".to_string()],
                                        generics: vec![],
                                        nullable: false,
                                        kind: dictionary::KindType::Primitive,
                                    },
                                    line: line.clone(),
                                    file: Some(file.to_string()),
                                }),
                                *line,
                            )?;

                            code.read(&value_var, GENERAL_REG2);
                            code.extend(&[
                                Allocate(GENERAL_REG1),
                                FillRange(GENERAL_REG2, GENERAL_REG1),
                                Move(POINTER_REG, GENERAL_REG1),
                            ]);
                            let mut return_kind = Kind {
                                body: TypeBody::Array {
                                    type_: Box::new(value.clone()),
                                    size: 0,
                                    refs: 0,
                                    nullable: false,
                                },
                                line: line.clone(),
                                file: Some(file.to_string()),
                            };
                            return Ok(Position::Value(return_kind));
                        } else {
                            unreachable!("array not handled properly by the compiler, please report this bug")
                        }
                    }
                },
                expression_parser::Literals::String(str) => {
                    let pos = new_const(context, &ConstValue::String(str.clone()))?;
                    code.push(ReadConst(pos, GENERAL_REG1));
                    return Ok(Position::Value(Kind {
                        body: TypeBody::Type {
                            refs: 0,
                            main: vec!["string".to_string()],
                            generics: vec![],
                            nullable: false,
                            kind: dictionary::KindType::Primitive,
                        },
                        line: line.clone(),
                        file: Some(file.to_string()),
                    }));
                }
                expression_parser::Literals::Char(c) => {
                    let pos = new_const(context, &ConstValue::Char(*c))?;
                    code.push(ReadConst(pos, GENERAL_REG1));
                    return Ok(Position::Value(Kind {
                        body: TypeBody::Type {
                            refs: 0,
                            main: vec!["char".to_string()],
                            generics: vec![],
                            nullable: false,
                            kind: dictionary::KindType::Primitive,
                        },
                        line: line.clone(),
                        file: Some(file.to_string()),
                    }));
                }
            }
        }
        Root::Parenthesis(val) => {
            let kind = expression(
                objects,
                val,
                scopes.unwrap(),
                code,
                context,
                &InnerPath {
                    file: file.to_string(),
                    block: None,
                    ident: "".to_string(),
                    kind: ImportKinds::Rd,
                },
                scope_len,
                None,
                *line,
            )?;
            return Ok(Position::Value(kind));
        }
    }
}

fn traverse_tail(
    objects: &mut Context,
    tail: &mut std::slice::Iter<'_, (expression_parser::TailNodes, Line)>,
    context: &mut runtime_types::Context,
    scopes: &mut Vec<ScopeCached>,
    code: &mut Code,
    fun: &InnerPath,
    pos: Position,
    scope_len: &mut usize,
) -> Result<Position, CodegenError> {
    use Instructions::*;
    let mut return_kind = Kind::void();
    match tail.next() {
        Some(node) => match &node.0 {
            expression_parser::TailNodes::Nested(ident) => match &pos {
                Position::Import(fname) => {
                    let root = identify_root(
                        objects,
                        &Root::Identifier(ident.to_string()),
                        Some(scopes),
                        &fname,
                        &node.1,
                        context,
                        code,
                        scope_len,
                        None,
                        &fun,
                    )?;
                    return traverse_tail(
                        objects, tail, context, scopes, code, fun, root, scope_len,
                    );
                }
                Position::BinImport(fname) => {
                    let root = identify_root(
                        objects,
                        &Root::Identifier(ident.to_string()),
                        Some(scopes),
                        &fname,
                        &node.1,
                        context,
                        code,
                        scope_len,
                        None,
                        &fun,
                    )?;
                    return traverse_tail(
                        objects, tail, context, scopes, code, fun, root, scope_len,
                    );
                }
                Position::Variable(vname, _kind) => {
                    let var = match find_var(&scopes, &vname) {
                        Some(var) => var,
                        None => Err(CodegenError::VariableNotFound(
                            vname.clone(),
                            node.1.clone(),
                        ))?,
                    };
                    let pos_cloned = var.pos.clone();
                    let kind = var.kind.as_ref().unwrap().clone();

                    match &kind.body {
                        TypeBody::Array {
                            type_,
                            size,
                            refs,
                            nullable,
                        } => {
                            let path = find_array_method(objects, &ident, &node.1)?;
                            code.read(&pos_cloned, RETURN_REG);
                            return traverse_tail(
                                objects,
                                tail,
                                context,
                                scopes,
                                code,
                                fun,
                                Position::Function(FunctionKind::Binary(path), *type_.clone()),
                                scope_len,
                            );
                        }
                        TypeBody::Type {
                            refs,
                            main,
                            generics,
                            nullable,
                            kind: kind_kind,
                        } => {
                            let _kind_main = match &_kind.body {
                                TypeBody::Type { refs, main, generics, nullable, kind: kind_kind } => main,
                                _ => unreachable!("kind not handled properly by the compiler, please report this bug"),
                            };
                            match kind_kind {
                                dictionary::KindType::Struct => {
                                    let structt = find_struct(
                                        objects,
                                        &_kind_main.first().unwrap(),
                                        &kind.file.as_ref().unwrap(),
                                    )
                                    .unwrap()
                                    .1;
                                    match structt.get_field(&ident) {
                                        Some((field, idx)) => match field {
                                            CompoundField::Field(ident) => {
                                                code.read(&pos_cloned, POINTER_REG);
                                                code.extend(&[
                                                    IndexStatic(idx + 1),
                                                    Move(POINTER_REG, GENERAL_REG1),
                                                ]);
                                                return_kind = structt.fields[idx].1.clone();
                                                *return_kind.refs_mut() += 1;
                                                let pos = traverse_tail(
                                                    objects,
                                                    tail,
                                                    context,
                                                    scopes,
                                                    code,
                                                    fun,
                                                    Position::CompoundField(
                                                        InnerPath {
                                                            file: kind.file.clone().unwrap(),
                                                            block: Some(
                                                                structt.identifier.to_string(),
                                                            ),
                                                            ident: main.first().unwrap().clone(),
                                                            kind: ImportKinds::Rd,
                                                        },
                                                        CompoundField::Field(ident.clone()),
                                                        kind,
                                                    ),
                                                    scope_len,
                                                );
                                                return pos;
                                            }
                                            CompoundField::Method(ident) => {
                                                code.read(&pos_cloned, GENERAL_REG1);
                                                return traverse_tail(
                                                    objects,
                                                    tail,
                                                    context,
                                                    scopes,
                                                    code,
                                                    fun,
                                                    Position::CompoundField(
                                                        InnerPath {
                                                            file: kind.file.clone().unwrap(),
                                                            block: Some(
                                                                main.last()
                                                                    .as_ref()
                                                                    .unwrap()
                                                                    .to_string(),
                                                            ),
                                                            ident: ident.clone(),
                                                            kind: ImportKinds::Rd,
                                                        },
                                                        CompoundField::Method(ident.clone()),
                                                        kind,
                                                    ),
                                                    scope_len,
                                                );
                                            }
                                            _ => todo!(),
                                        },
                                        None => Err(CodegenError::FieldNotInStruct(
                                            ident.clone(),
                                            node.1.clone(),
                                        ))?,
                                    };
                                }
                                dictionary::KindType::UserData => {
                                    let userdata = find_userdata(
                                        objects,
                                        &_kind_main.first().unwrap(),
                                        &kind.file.as_ref().unwrap(),
                                    )
                                    .unwrap()
                                    .1;
                                    match userdata.get_field(&ident) {
                                        Some((field, idx)) => match field {
                                            CompoundField::Method(ident) => {
                                                code.read(&pos_cloned, GENERAL_REG1);
                                                return traverse_tail(
                                                    objects,
                                                    tail,
                                                    context,
                                                    scopes,
                                                    code,
                                                    fun,
                                                    Position::CompoundField(
                                                        InnerPath {
                                                            file: kind.file.clone().unwrap(),
                                                            block: Some(
                                                                main.last()
                                                                    .as_ref()
                                                                    .unwrap()
                                                                    .to_string(),
                                                            ),
                                                            ident: ident.clone(),
                                                            kind: ImportKinds::Dll,
                                                        },
                                                        CompoundField::Method(ident.clone()),
                                                        kind,
                                                    ),
                                                    scope_len,
                                                );
                                            }
                                            _ => todo!(),
                                        },
                                        None => Err(CodegenError::FieldNotInStruct(
                                            ident.clone(),
                                            node.1.clone(),
                                        ))?,
                                    };
                                }
                                // in case of primitives we have to use internal functions in the core library
                                dictionary::KindType::Primitive => {
                                    let path =
                                        find_primitive_method(objects, &kind, &ident, &node.1)?;
                                    code.push(Move(GENERAL_REG1, RETURN_REG));
                                    return traverse_tail(
                                        objects,
                                        tail,
                                        context,
                                        scopes,
                                        code,
                                        fun,
                                        Position::Function(FunctionKind::Binary(path), kind),
                                        scope_len,
                                    );
                                }
                                dictionary::KindType::Enum => todo!(),
                                dictionary::KindType::Trait => todo!(),
                                dictionary::KindType::Fun => todo!(),
                                dictionary::KindType::Error => todo!(),
                                dictionary::KindType::BinFun => todo!(),
                                dictionary::KindType::SelfRef => todo!(),
                                dictionary::KindType::None => todo!(),
                            }
                        }
                        _ => todo!(),
                    }

                    /*if kind.array_depth > 0 {
                        let path = find_array_method(objects, &ident, &node.1)?;
                        code.push(Move(GENERAL_REG1, RETURN_REG));
                        return traverse_tail(
                            objects,
                            tail,
                            context,
                            scopes,
                            code,
                            fun,
                            Position::Function(FunctionKind::Binary(path), kind),
                            scope_len,
                        );
                    }*/
                }
                Position::CompoundField(path, field, kind) => {
                    let structt = find_struct(objects, &path.ident, &path.file);
                }
                Position::Function(_, kind) => {
                    Err(CodegenError::CannotAttachMethodsToFunctions(node.1.clone()))?
                }
                Position::Pointer(ptr) => {
                    todo!()
                }
                Position::Compound(_kind) => {
                    let kind_main = match &_kind.body {
                        TypeBody::Type {
                            refs,
                            main,
                            generics,
                            nullable,
                            kind: kind_kind,
                        } => main,
                        _ => unreachable!(
                            "kind not handled properly by the compiler, please report this bug"
                        ),
                    };
                    if let Some((name, structt)) = find_struct(
                        objects,
                        &kind_main.first().unwrap(),
                        &_kind.file.as_ref().unwrap(),
                    ) {
                        let field = match structt.fields.iter().find(|field| &field.0 == ident) {
                            Some(field) => field,
                            None => Err(CodegenError::FieldNotInStruct(
                                ident.clone(),
                                node.1.clone(),
                            ))?,
                        };
                    }
                    todo!()
                }
                Position::Value(kind) => {
                    let kind = correct_kind(objects, kind, fun, &node.1)?;
                    match &kind.body {
                        TypeBody::Array {
                            type_,
                            size,
                            refs,
                            nullable,
                        } => {
                            let path = find_array_method(objects, &ident, &node.1)?;
                            code.push(Move(GENERAL_REG1, RETURN_REG));
                            return traverse_tail(
                                objects,
                                tail,
                                context,
                                scopes,
                                code,
                                fun,
                                Position::Function(FunctionKind::Binary(path), *type_.clone()),
                                scope_len,
                            );
                        }
                        TypeBody::Type {
                            refs,
                            main,
                            generics,
                            nullable,
                            kind: kind_kind,
                        } => {
                            let _kind_main = match &kind.body {
                                TypeBody::Type { refs, main, generics, nullable, kind: kind_kind } => main,
                                _ => unreachable!("kind not handled properly by the compiler, please report this bug"),
                            };
                            match kind_kind {
                                dictionary::KindType::Struct => {
                                    let structt = find_struct(
                                        objects,
                                        &_kind_main.first().unwrap(),
                                        &kind.file.as_ref().unwrap(),
                                    )
                                    .unwrap()
                                    .1;
                                    match structt.get_field(&ident) {
                                        Some((field, idx)) => match field {
                                            CompoundField::Field(ident) => {
                                                code.extend(&[
                                                    IndexStatic(idx + 1),
                                                    Move(POINTER_REG, GENERAL_REG1),
                                                ]);
                                                return_kind = structt.fields[idx].1.clone();
                                                *return_kind.refs_mut() += 1;
                                                let pos = traverse_tail(
                                                    objects,
                                                    tail,
                                                    context,
                                                    scopes,
                                                    code,
                                                    fun,
                                                    Position::CompoundField(
                                                        InnerPath {
                                                            file: kind.file.clone().unwrap(),
                                                            block: Some(
                                                                structt.identifier.to_string(),
                                                            ),
                                                            ident: main.first().unwrap().clone(),
                                                            kind: ImportKinds::Rd,
                                                        },
                                                        CompoundField::Field(ident.clone()),
                                                        kind,
                                                    ),
                                                    scope_len,
                                                );
                                                return pos;
                                            }
                                            CompoundField::Method(ident) => {
                                                return traverse_tail(
                                                    objects,
                                                    tail,
                                                    context,
                                                    scopes,
                                                    code,
                                                    fun,
                                                    Position::CompoundField(
                                                        InnerPath {
                                                            file: kind.file.clone().unwrap(),
                                                            block: Some(
                                                                main.last()
                                                                    .as_ref()
                                                                    .unwrap()
                                                                    .to_string(),
                                                            ),
                                                            ident: ident.clone(),
                                                            kind: ImportKinds::Rd,
                                                        },
                                                        CompoundField::Method(ident.clone()),
                                                        kind,
                                                    ),
                                                    scope_len,
                                                );
                                            }
                                            _ => todo!(),
                                        },
                                        None => Err(CodegenError::FieldNotInStruct(
                                            ident.clone(),
                                            node.1.clone(),
                                        ))?,
                                    };
                                }
                                dictionary::KindType::UserData => {
                                    let userdata = find_userdata(
                                        objects,
                                        &_kind_main.first().unwrap(),
                                        &kind.file.as_ref().unwrap(),
                                    )
                                    .unwrap()
                                    .1;
                                    match userdata.get_field(&ident) {
                                        Some((field, idx)) => match field {
                                            CompoundField::Method(ident) => {
                                                return traverse_tail(
                                                    objects,
                                                    tail,
                                                    context,
                                                    scopes,
                                                    code,
                                                    fun,
                                                    Position::CompoundField(
                                                        InnerPath {
                                                            file: kind.file.clone().unwrap(),
                                                            block: Some(
                                                                main.last()
                                                                    .as_ref()
                                                                    .unwrap()
                                                                    .to_string(),
                                                            ),
                                                            ident: ident.clone(),
                                                            kind: ImportKinds::Dll,
                                                        },
                                                        CompoundField::Method(ident.clone()),
                                                        kind,
                                                    ),
                                                    scope_len,
                                                );
                                            }
                                            _ => todo!(),
                                        },
                                        None => Err(CodegenError::FieldNotInStruct(
                                            ident.clone(),
                                            node.1.clone(),
                                        ))?,
                                    };
                                }
                                // in case of primitives we have to use internal functions in the core library
                                dictionary::KindType::Primitive => {
                                    let path =
                                        find_primitive_method(objects, &kind, &ident, &node.1)?;
                                    code.push(Move(GENERAL_REG1, RETURN_REG));
                                    return traverse_tail(
                                        objects,
                                        tail,
                                        context,
                                        scopes,
                                        code,
                                        fun,
                                        Position::Function(FunctionKind::Binary(path), kind),
                                        scope_len,
                                    );
                                }
                                dictionary::KindType::Enum => todo!(),
                                dictionary::KindType::Trait => todo!(),
                                dictionary::KindType::Fun => todo!(),
                                dictionary::KindType::Error => todo!(),
                                dictionary::KindType::BinFun => todo!(),
                                dictionary::KindType::SelfRef => todo!(),
                                dictionary::KindType::None => todo!(),
                            }
                        }
                        _ => todo!(),
                    }
                }
            },
            expression_parser::TailNodes::Index(idx) => match &pos {
                Position::BinImport(_) => Err(CodegenError::CannotIndexFile(node.1.clone()))?,
                Position::Function(_, _) => Err(CodegenError::CannotIndexFunction(node.1.clone()))?,
                Position::CompoundField(path, field, ptr) => {
                    let ident = if let CompoundField::Field(ident) = field {
                        ident
                    } else {
                        Err(CodegenError::CannotIndexNonArray(
                            pos.clone(),
                            node.1.clone(),
                        ))?
                    };
                    let field_kind = {
                        if let Some(structt) = find_struct(objects, &path.ident, &path.file) {
                            let field =
                                match structt.1.fields.iter().find(|field| &field.0 == ident) {
                                    Some(field) => field,
                                    None => Err(CodegenError::FieldNotInStruct(
                                        ident.clone(),
                                        node.1.clone(),
                                    ))?,
                                };
                            field.1.clone()
                        } else {
                            Err(CodegenError::FieldNotInStruct(
                                ident.clone(),
                                node.1.clone(),
                            ))?
                        }
                    };
                    // first dereference the pointer
                    code.push(ReadPtr(GENERAL_REG1));

                    let mut return_kind = match field_kind.body {
                        TypeBody::Array {
                            type_,
                            size,
                            refs,
                            nullable,
                        } => *type_.clone(),
                        _ => Err(CodegenError::CannotIndexNonArray(
                            pos.clone(),
                            node.1.clone(),
                        ))?,
                    };

                    // save the pointer from GENERAL_REG1 to stack
                    *scope_len += 1;
                    let obj = create_var_pos(scopes);
                    let scopes_len = scopes.len();
                    scopes[scopes_len - 1].variables.insert(
                        scope_len.to_string(),
                        Variable {
                            kind: Some(return_kind.clone()),
                            pos: obj.clone(),
                            value: None,
                            line: node.1.clone(),
                        },
                    );
                    code.write(GENERAL_REG1, &obj);

                    expression(
                        objects,
                        idx,
                        scopes,
                        code,
                        context,
                        &fun,
                        scope_len,
                        Some(Kind {
                            body: TypeBody::Type {
                                refs: 0,
                                main: vec!["uint".to_string()],
                                generics: vec![],
                                nullable: false,
                                kind: dictionary::KindType::Primitive,
                            },
                            line: node.1.clone(),
                            file: Some(fun.file.clone()),
                        }),
                        node.1,
                    )?;

                    // restore the pointer from stack to POINTER_REG
                    code.read(&obj, POINTER_REG);
                    code.extend(&[Index(GENERAL_REG1), Move(POINTER_REG, GENERAL_REG1)]);
                    let pos = Position::Pointer(return_kind);
                    return traverse_tail(
                        objects, tail, context, scopes, code, fun, pos, scope_len,
                    );
                }
                Position::Import(_) => Err(CodegenError::CannotIndexFile(node.1.clone()))?,
                Position::Variable(var, _) => {
                    let var = match find_var(scopes, &var) {
                        Some(var) => var,
                        None => Err(CodegenError::VariableNotFound(var.clone(), node.1.clone()))?,
                    };
                    let pos_cloned = var.pos.clone();

                    return_kind = match &var.kind.as_ref().unwrap().body {
                        TypeBody::Array {
                            type_,
                            size,
                            refs,
                            nullable,
                        } => *type_.clone(),
                        _ => Err(CodegenError::CannotIndexNonArray(
                            pos.clone(),
                            node.1.clone(),
                        ))?,
                    };

                    expression(
                        objects,
                        idx,
                        scopes,
                        code,
                        context,
                        &fun,
                        scope_len,
                        Some(Kind {
                            body: TypeBody::Type {
                                refs: 0,
                                main: vec!["uint".to_string()],
                                generics: vec![],
                                nullable: false,
                                kind: dictionary::KindType::Primitive,
                            },
                            line: node.1.clone(),
                            file: Some(fun.file.clone()),
                        }),
                        node.1,
                    )?;

                    code.read(&pos_cloned, POINTER_REG);
                    code.extend(&[Index(GENERAL_REG1), Move(POINTER_REG, GENERAL_REG1)]);

                    let pos = Position::Pointer(return_kind);
                    return traverse_tail(
                        objects, tail, context, scopes, code, fun, pos, scope_len,
                    );
                }
                Position::Pointer(ptr) => {
                    let mut return_kind = match &ptr.body {
                        TypeBody::Array {
                            type_,
                            size,
                            refs,
                            nullable,
                        } => *type_.clone(),
                        _ => Err(CodegenError::CannotIndexNonArray(
                            pos.clone(),
                            node.1.clone(),
                        ))?,
                    };

                    // deref once
                    code.push(ReadPtr(GENERAL_REG1));

                    // save the pointer from GENERAL_REG1 to stack
                    *scope_len += 1;
                    let obj = create_var_pos(scopes);
                    let scopes_len = scopes.len();
                    scopes[scopes_len - 1].variables.insert(
                        scope_len.to_string(),
                        Variable {
                            kind: Some(return_kind.clone()),
                            pos: obj.clone(),
                            value: None,
                            line: node.1.clone(),
                        },
                    );
                    code.write(GENERAL_REG1, &obj);

                    expression(
                        objects,
                        idx,
                        scopes,
                        code,
                        context,
                        &fun,
                        scope_len,
                        Some(Kind {
                            body: TypeBody::Type {
                                refs: 0,
                                main: vec!["uint".to_string()],
                                generics: vec![],
                                nullable: false,
                                kind: dictionary::KindType::Primitive,
                            },
                            line: node.1.clone(),
                            file: Some(fun.file.clone()),
                        }),
                        node.1,
                    )?;

                    // restore the pointer from stack to POINTER_REG
                    code.read(&obj, POINTER_REG);
                    code.extend(&[Index(GENERAL_REG1), Move(POINTER_REG, GENERAL_REG1)]);
                    let pos = Position::Pointer(return_kind);
                    return traverse_tail(
                        objects, tail, context, scopes, code, fun, pos, scope_len,
                    );
                }
                Position::Compound(_) => Err(CodegenError::CannotIndexNonArray(
                    pos.clone(),
                    node.1.clone(),
                ))?,
                Position::Value(ptr) => {
                    let mut return_kind = match &ptr.body {
                        TypeBody::Array {
                            type_,
                            size,
                            refs,
                            nullable,
                        } => *type_.clone(),
                        _ => Err(CodegenError::CannotIndexNonArray(
                            pos.clone(),
                            node.1.clone(),
                        ))?,
                    };

                    // save the pointer from GENERAL_REG1 to stack
                    *scope_len += 1;
                    let obj = create_var_pos(scopes);
                    let scopes_len = scopes.len();
                    scopes[scopes_len - 1].variables.insert(
                        scope_len.to_string(),
                        Variable {
                            kind: Some(return_kind.clone()),
                            pos: obj.clone(),
                            value: None,
                            line: node.1.clone(),
                        },
                    );
                    code.write(GENERAL_REG1, &obj);

                    expression(
                        objects,
                        idx,
                        scopes,
                        code,
                        context,
                        &fun,
                        scope_len,
                        Some(Kind {
                            body: TypeBody::Type {
                                refs: 0,
                                main: vec!["uint".to_string()],
                                generics: vec![],
                                nullable: false,
                                kind: dictionary::KindType::Primitive,
                            },
                            line: node.1.clone(),
                            file: Some(fun.file.clone()),
                        }),
                        node.1,
                    )?;

                    // restore the pointer from stack to POINTER_REG
                    code.read(&obj, POINTER_REG);
                    code.extend(&[Index(GENERAL_REG1), Move(POINTER_REG, GENERAL_REG1)]);
                    let pos = Position::Pointer(return_kind);
                    return traverse_tail(
                        objects, tail, context, scopes, code, fun, pos, scope_len,
                    );
                }
            },
            expression_parser::TailNodes::Call(call_params) => match &pos {
                Position::Function(fun_kind, _) => {
                    match fun_kind {
                        FunctionKind::Fun(fun_path) => {
                            let mut fun_kind = call_fun(
                                objects,
                                fun_path,
                                context,
                                scopes,
                                code,
                                scope_len,
                                call_params,
                                &node.1,
                                fun,
                            )?;
                            fun_kind.file = Some(fun_path.file.clone());
                            let pos = Position::Value(fun_kind);
                            return traverse_tail(
                                objects, tail, context, scopes, code, fun, pos, scope_len,
                            );
                        }
                        FunctionKind::Binary(fun_path) => {
                            let mut kind = call_binary(
                                objects,
                                fun_path,
                                context,
                                scopes,
                                code,
                                scope_len,
                                call_params,
                                &node.1,
                                &fun,
                            )?;
                            kind.file = Some(fun_path.file.clone());
                            let pos = Position::Value(kind);
                            return traverse_tail(
                                objects, tail, context, scopes, code, fun, pos, scope_len,
                            );
                        }
                        FunctionKind::Dynamic(fun) => todo!("dynamic function call"),
                    };
                }
                Position::Compound(kind) => {
                    let (constructor, kind_kind) = match &kind.body {
                        TypeBody::Type {
                            refs,
                            main,
                            generics,
                            nullable,
                            kind: kind_kind,
                        } => (main.last().unwrap(), kind_kind),
                        _ => unreachable!(
                            "kind not handled properly by the compiler, please report this bug"
                        ),
                    };
                    let import_kind = match kind_kind.to_import_kind() {
                        Some(kind) => kind,
                        None => Err(CodegenError::CannotCallType(kind.clone(), node.1.clone()))?,
                    };
                    let path = InnerPath {
                        file: kind.file.as_ref().unwrap().to_string(),
                        block: Some(constructor.to_string()),
                        ident: constructor.to_string(),
                        kind: import_kind,
                    };
                    let mut kind = call_whichever(
                        objects,
                        &path,
                        context,
                        scopes,
                        code,
                        scope_len,
                        call_params,
                        &node.1,
                        fun,
                    )?;
                    kind.file = Some(path.file.clone());
                    let pos = Position::Value(kind);
                    return traverse_tail(
                        objects, tail, context, scopes, code, fun, pos, scope_len,
                    );
                }
                Position::CompoundField(path, field, kind) => {
                    match field {
                        CompoundField::Method(ident) => ident,
                        _ => todo!(),
                    };
                    code.push(Move(GENERAL_REG1, RETURN_REG));
                    let mut kind = call_whichever(
                        objects,
                        path,
                        context,
                        scopes,
                        code,
                        scope_len,
                        call_params,
                        &node.1,
                        fun,
                    )?;
                    kind.file = Some(path.file.clone());
                    let pos = Position::Value(kind);
                    return traverse_tail(
                        objects, tail, context, scopes, code, fun, pos, scope_len,
                    );
                }
                _ => Err(CodegenError::CanCallOnlyFunctions(node.1.clone()))?,
            },
            expression_parser::TailNodes::Cast(_) => todo!(),
        },
        None => {
            // finish the sequence
        }
    }
    Ok(pos)
}

fn find_primitive_method(
    objects: &mut Context,
    kind: &Kind,
    method: &str,
    line: &Line,
) -> Result<InnerPath, CodegenError> {
    match &kind.body {
        TypeBody::Type {
            refs,
            main,
            generics,
            nullable,
            kind: kind_kind,
        } => {
            let name = match kind_kind {
                dictionary::KindType::Primitive => {
                    format!("{}{}", main.first().unwrap(), method)
                }
                _ => Err(CodegenError::CannotCallType(kind.clone(), line.clone()))?,
            };
            let path = InnerPath {
                file: "#core".to_string(),
                block: None,
                ident: name,
                kind: ImportKinds::Dll,
            };
            Ok(path)
        }
        _ => Err(CodegenError::CannotCallType(kind.clone(), line.clone()))?,
    }
}

fn find_array_method(
    objects: &mut Context,
    method: &str,
    line: &Line,
) -> Result<InnerPath, CodegenError> {
    let name = format!("arr{}", method);
    let path = InnerPath {
        file: "#core".to_string(),
        block: None,
        ident: name,
        kind: ImportKinds::Dll,
    };
    Ok(path)
}

fn call_binary(
    objects: &mut Context,
    fun: &InnerPath,
    context: &mut runtime_types::Context,
    scopes: &mut Vec<ScopeCached>,
    code: &mut Code,
    scope_len: &mut usize,
    call_params: &FunctionCall,
    line: &Line,
    this: &InnerPath,
) -> Result<Kind, CodegenError> {
    use Instructions::*;
    let lib_id = objects.1.get(&fun.file).unwrap().id;
    let mut temp_code = Code { code: Vec::new() };
    // setup arguments (stack is not needed)
    let args_len = fun.get_bin(objects)?.args.len();
    let mut args = Vec::new();
    *scope_len += 1;
    let obj = create_var_pos(scopes);
    let scopes_len = scopes.len();
    scopes[scopes_len - 1].variables.insert(
        scope_len.to_string(),
        Variable {
            kind: Some(Kind::void()),
            pos: obj.clone(),
            value: None,
            line: line.clone(),
        },
    );
    let takes_self = fun.get_bin(objects)?.takes_self;
    let called_fun = fun.get_bin(objects)?;
    temp_code.extend(&[
        Freeze,
        AllocateStatic(called_fun.args.len().max(call_params.args.len()) + takes_self as usize),
    ]);
    temp_code.write(POINTER_REG, &obj);
    if called_fun.args.len() != call_params.args.len() {
        Err(CodegenError::IncorrectNumberOfArgs(
            called_fun.args.len(),
            call_params.args.len(),
            line.clone(),
        ))?;
    }
    if takes_self {
        temp_code.push(Move(RETURN_REG, GENERAL_REG1));
        temp_code.extend(&[IndexStatic(0), WritePtr(GENERAL_REG1)])
    }
    for (idx, arg) in call_params
        .args
        .iter()
        .zip(called_fun.args.clone())
        .enumerate()
    {
        let kind = expression(
            objects,
            arg.0,
            scopes,
            &mut temp_code,
            context,
            &this,
            scope_len,
            Some(arg.1.clone().1),
            arg.0.get_line(),
        )?;
        args.push(kind);
        temp_code.read(&obj, POINTER_REG);
        temp_code.extend(&[
            IndexStatic(idx + takes_self as usize),
            WritePtr(GENERAL_REG1),
        ])
    }
    // type check
    for (idx, arg) in args.iter().enumerate() {
        if idx >= args_len {
            Err(CodegenError::IncorrectArgs(line.clone()))?;
        }
        let cmp = fun.get_bin(objects)?.args[idx].1.cmp(&arg);
        if cmp.is_not_equal() {
            let expected = fun.get_bin(objects)?.args[idx].1.clone();
            let ident = fun.get_bin(objects)?.args[idx].0.clone();
            return Err(CodegenError::ArgTypeMismatch(
                Arg {
                    identifier: ident,
                    kind: expected,
                    line: fun.get_bin(objects)?.args[idx].3.clone(),
                },
                arg.clone(),
                cmp,
                line.clone(),
            ));
        }
    }
    temp_code.read(&obj, ARGS_REG);
    let called_fun = fun.get_bin(objects)?;
    // call
    temp_code.extend(&[
        Cal(lib_id, called_fun.assign),
        Unfreeze,
        Move(RETURN_REG, GENERAL_REG1),
    ]);
    merge_code(&mut code.code, &temp_code.code, *scope_len);
    Ok(called_fun.return_type.clone())
}

fn call_fun(
    objects: &mut Context,
    fun: &InnerPath,
    context: &mut runtime_types::Context,
    scopes: &mut Vec<ScopeCached>,
    code: &mut Code,
    scope_len: &mut usize,
    call_params: &FunctionCall,
    line: &Line,
    this: &InnerPath,
) -> Result<Kind, CodegenError> {
    use Instructions::*;
    let mut temp_code = Code { code: Vec::new() };
    let called_fun = fun.get(objects)?;
    *scope_len += 1;
    let obj = create_var_pos(scopes);
    let scopes_len = scopes.len();
    scopes[scopes_len - 1].variables.insert(
        scope_len.to_string(),
        Variable {
            kind: Some(Kind::void()),
            pos: obj.clone(),
            value: None,
            line: line.clone(),
        },
    );
    let takes_self = called_fun.takes_self;
    temp_code.extend(&[
        Freeze,
        AllocateStatic(
            called_fun
                .args
                .len()
                .max(call_params.args.len() + takes_self as usize),
        ),
    ]);
    temp_code.write(POINTER_REG, &obj);
    // setup args
    if called_fun.args.len() != call_params.args.len() {
        Err(CodegenError::IncorrectNumberOfArgs(
            called_fun.args.len(),
            call_params.args.len(),
            line.clone(),
        ))?;
    }
    if takes_self {
        temp_code.push(Move(RETURN_REG, GENERAL_REG1));
        temp_code.extend(&[IndexStatic(0), WritePtr(GENERAL_REG1)])
    }
    let mut args = Vec::new();
    for (idx, arg) in call_params
        .args
        .iter()
        .zip(called_fun.args.clone())
        .enumerate()
    {
        let kind = expression(
            objects,
            arg.0,
            scopes,
            &mut temp_code,
            context,
            &fun,
            scope_len,
            Some(arg.1.clone().kind),
            arg.1.line,
        )?;
        args.push(kind);
        temp_code.read(&obj, POINTER_REG);
        temp_code.extend(&[
            IndexStatic(idx + takes_self as usize),
            WritePtr(GENERAL_REG1),
        ])
    }
    // type check
    let called_fun = fun.get(objects)?;
    for (idx, arg) in args.iter().enumerate() {
        if idx >= called_fun.args.len() {
            Err(CodegenError::IncorrectArgs(line.clone()))?;
        }
        let cmp = called_fun.args[idx].kind.cmp(&arg);
        if cmp.is_not_equal() {
            return Err(CodegenError::ArgTypeMismatch(
                called_fun.args[idx].clone(),
                arg.clone(),
                cmp,
                line.clone(),
            ));
        }
    }
    temp_code.read(&obj, ARGS_REG);
    // call
    temp_code.extend(&[
        // The function may not be generated yet, so we need to jump to its id
        // which will be replaced with the actual location later
        Jump(called_fun.id),
        Unfreeze,
        Move(RETURN_REG, GENERAL_REG1),
    ]);
    merge_code(&mut code.code, &temp_code.code, *scope_len);
    Ok(called_fun.return_type.clone().unwrap_or(Kind {
        body: TypeBody::Type {
            refs: 0,
            main: vec!["null".to_string()],
            generics: vec![],
            nullable: false,
            kind: dictionary::KindType::Primitive,
        },
        line: line.clone(),
        file: Some(fun.file.clone()),
    }))
}

fn call_whichever(
    objects: &mut Context,
    fun: &InnerPath,
    context: &mut runtime_types::Context,
    scopes: &mut Vec<ScopeCached>,
    code: &mut Code,
    scope_len: &mut usize,
    call_params: &FunctionCall,
    line: &Line,
    this: &InnerPath,
) -> Result<Kind, CodegenError> {
    match fun.kind {
        ImportKinds::Dll => {
            let kind = call_binary(
                objects,
                fun,
                context,
                scopes,
                code,
                scope_len,
                call_params,
                line,
                this,
            )?;
            return Ok(kind);
        }
        ImportKinds::Rd => {
            let kind = call_fun(
                objects,
                fun,
                context,
                scopes,
                code,
                scope_len,
                call_params,
                line,
                this,
            )?;
            return Ok(kind);
        }
    }
}

fn get_scope(
    objects: &mut Context,
    block: &Vec<Nodes>,
    context: &mut runtime_types::Context,
    other_scopes: &mut Vec<ScopeCached>,
    code: &mut Code,
    fun: &InnerPath,
) -> Result<(usize, ScopeTerminator), CodegenError> {
    let mut max_scope_len = 0;
    other_scopes.push(ScopeCached {
        variables: HashMap::new(),
    });
    macro_rules! last {
        ($arr: expr) => {{
            let len = $arr.len();
            &mut $arr[len - 1]
        }};
    }

    macro_rules! open_scope {
        ($block: expr, $code: expr) => {{
            let (scope_len, terminator) =
                get_scope(objects, $block, context, other_scopes, $code, fun)?;
            other_scopes.pop();
            max_scope_len += scope_len;
            (scope_len, terminator)
        }};
    }

    let bool_type = Kind {
        body: TypeBody::Type {
            refs: 0,
            main: vec!["bool".to_string()],
            generics: vec![],
            nullable: false,
            kind: dictionary::KindType::Primitive,
        },
        line: Line { line: 0, column: 0 },
        file: Some(fun.file.clone()),
    };

    for node in block {
        match node {
            crate::codeblock_parser::Nodes::Let {
                ident,
                expr,
                kind,
                line,
            } => {
                if let Some(var) = find_var(other_scopes, ident) {
                    Err(CodegenError::VariableAlreadyDeclared(
                        ident.clone(),
                        line.clone(),
                        var.line.clone(),
                    ))?;
                }
                let (expr_kind, pos) = match expr {
                    Some(expr) => {
                        let expr_kind = expression(
                            objects,
                            expr,
                            other_scopes,
                            code,
                            context,
                            &fun,
                            &mut max_scope_len,
                            kind.clone(),
                            *line,
                        )?;
                        max_scope_len += 1;
                        let pos = create_var_pos(&other_scopes);
                        let cache = last!(other_scopes);
                        cache.variables.insert(
                            ident.clone(),
                            Variable {
                                kind: None,
                                pos,
                                value: try_get_const_val(),
                                line: line.clone(),
                            },
                        );
                        code.write(GENERAL_REG1, &pos);
                        (expr_kind, pos)
                    }
                    None => {
                        if let Some(kind) = kind {
                            if !kind.get_nullable() {
                                Err(CodegenError::TypeNotNullable(line.clone()))?;
                            }
                        } else {
                            Err(CodegenError::NotInitializedNoType(line.clone()))?;
                        }
                        let null = new_const(context, &ConstValue::Null)?;
                        use Instructions::*;
                        max_scope_len += 1;
                        let pos = create_var_pos(&other_scopes);
                        let cache = last!(other_scopes);
                        cache.variables.insert(
                            ident.clone(),
                            Variable {
                                kind: None,
                                pos,
                                value: try_get_const_val(),
                                line: line.clone(),
                            },
                        );
                        code.extend(&[ReadConst(null, GENERAL_REG1)]);
                        code.write(GENERAL_REG1, &pos);
                        (
                            Kind {
                                body: TypeBody::Type {
                                    refs: 0,
                                    main: vec!["null".to_string()],
                                    generics: vec![],
                                    nullable: false,
                                    kind: dictionary::KindType::Primitive,
                                },
                                line: line.clone(),
                                file: Some(fun.file.clone()),
                            },
                            pos,
                        )
                    }
                };
                let kind = match kind {
                    Some(kind) => kind.clone(),
                    None => expr_kind.clone(),
                };
                let kind = match correct_kind(&objects, &kind, &fun, &line) {
                    Ok(kind) => kind,
                    Err(e) => {
                        return Err(e);
                    }
                };
                match cast(
                    objects,
                    &expr_kind,
                    &kind,
                    code,
                    context,
                    &fun,
                    &line,
                    GENERAL_REG1,
                ) {
                    Some(_) => {
                        code.write(GENERAL_REG1, &pos);
                    }
                    None => {
                        return Err(CodegenError::CouldNotCastTo(expr_kind, kind, line.clone()));
                    }
                }
                let cache = last!(other_scopes);
                cache.variables.get_mut(ident).unwrap().kind = Some(kind);
            }
            crate::codeblock_parser::Nodes::If {
                cond,
                body,
                elif,
                els,
                line,
            } => {
                use Instructions::*;
                let mut all_end_with_return = true;
                // if
                let mut expr_code = Code { code: Vec::new() };
                let mut block_code = Code { code: Vec::new() };
                let mut gotos = Vec::new();
                expression(
                    objects,
                    cond,
                    other_scopes,
                    &mut expr_code,
                    context,
                    &fun,
                    &mut max_scope_len,
                    Some(bool_type.clone()),
                    *line,
                )?;
                let (scope, terminator) = open_scope!(body, &mut block_code);
                if terminator != ScopeTerminator::Return {
                    all_end_with_return = false;
                }
                expr_code.push(Branch(
                    expr_code.code.len() + 1,
                    expr_code.code.len() + block_code.code.len() + 2,
                ));
                // elifs
                let mut elifs = Vec::new();
                for elif in elif {
                    let mut expr_code = Code { code: Vec::new() };
                    let mut block_code = Code { code: Vec::new() };
                    expression(
                        objects,
                        &elif.0,
                        other_scopes,
                        &mut expr_code,
                        context,
                        &fun,
                        &mut max_scope_len,
                        Some(bool_type.clone()),
                        *line,
                    )?;
                    let (scope, terminator) = open_scope!(&elif.1, &mut block_code);
                    if terminator != ScopeTerminator::Return {
                        all_end_with_return = false;
                    }
                    expr_code.push(Branch(
                        expr_code.code.len() + 1,
                        expr_code.code.len() + block_code.code.len() + 2,
                    ));
                    elifs.push((expr_code, block_code, scope));
                }
                // else
                let elsse = match els {
                    Some(els) => {
                        let mut block_code = Code { code: Vec::new() };
                        let (scope, terminator) = open_scope!(&els.0, &mut block_code);
                        if terminator != ScopeTerminator::Return {
                            all_end_with_return = false;
                        }
                        (block_code, scope)
                    }
                    None => (Code { code: Vec::new() }, 0),
                };
                // merge
                let mut buffer = Vec::new();
                block_code.push(Goto(0));
                merge_code(&mut buffer, &expr_code.code, scope);
                merge_code(&mut buffer, &block_code.code, scope);
                gotos.push(buffer.len() - 1);
                for (expr_code, mut block_code, scope) in elifs {
                    block_code.push(Goto(0));
                    merge_code(&mut buffer, &expr_code.code, scope);
                    merge_code(&mut buffer, &block_code.code, scope);
                    gotos.push(buffer.len() - 1);
                }
                merge_code(&mut buffer, &elsse.0.code, elsse.1);
                // fix gotos
                for goto in &gotos {
                    buffer[*goto] = Goto(buffer.len());
                }
                // append
                merge_code(&mut code.code, &buffer, scope);
                if all_end_with_return && els.is_some() {
                    return Ok((max_scope_len, ScopeTerminator::Return));
                }
            }
            crate::codeblock_parser::Nodes::While {
                cond,
                body,
                line,
                ident,
            } => {
                use Instructions::*;
                let mut expr_code = Code { code: Vec::new() };
                let mut block_code = Code { code: Vec::new() };
                let kind = expression(
                    objects,
                    cond,
                    other_scopes,
                    &mut expr_code,
                    context,
                    &fun,
                    &mut max_scope_len,
                    Some(bool_type.clone()),
                    *line,
                )?;
                if kind.cmp(&bool_type).is_not_equal() {
                    return Err(CodegenError::ExpectedBool(line.clone()));
                }
                let (scope, terminator) = open_scope!(body, &mut block_code);
                expr_code.push(Branch(
                    expr_code.code.len() + 1,
                    expr_code.code.len() + block_code.code.len() + 2,
                ));
                let mut buffer = Vec::new();
                merge_code(&mut buffer, &expr_code.code, scope);
                merge_code(&mut buffer, &block_code.code, scope);
                buffer.push(Goto(0));
                merge_code(&mut code.code, &buffer, scope);
                if terminator != ScopeTerminator::None {
                    return Ok((max_scope_len, ScopeTerminator::Return));
                }
            }
            crate::codeblock_parser::Nodes::For {
                ident,
                expr,
                body,
                line,
                ident2,
            } => todo!(),
            crate::codeblock_parser::Nodes::Return { expr, line } => {
                use Instructions::*;
                let mut expr_code = Code { code: Vec::new() };
                let kind = match expr {
                    Some(expr) => {
                        let kind = expression(
                            objects,
                            expr,
                            other_scopes,
                            &mut expr_code,
                            context,
                            &fun,
                            &mut max_scope_len,
                            fun.get(objects)?.return_type.clone(),
                            *line,
                        )?;
                        expr_code.push(Move(GENERAL_REG1, RETURN_REG));
                        kind
                    }
                    None => {
                        expr_code.push(ReadConst(0, RETURN_REG));
                        Kind {
                            body: TypeBody::Type {
                                refs: 0,
                                main: vec!["null".to_string()],
                                generics: vec![],
                                nullable: false,
                                kind: dictionary::KindType::Primitive,
                            },
                            line: line.clone(),
                            file: Some(fun.file.clone()),
                        }
                    }
                };
                let this_fun = fun.get(objects)?;
                if let Some(ret_type) = &this_fun.return_type {
                    let cmp = ret_type.cmp(&kind);
                    if cmp.is_not_equal() {
                        return Err(CodegenError::VariableTypeMismatch(
                            ret_type.clone(),
                            kind,
                            cmp,
                            line.clone(),
                        ));
                    }
                } else if !kind.is_null() {
                    return Err(CodegenError::VariableTypeMismatch(
                        Kind {
                            body: TypeBody::Type {
                                refs: 0,
                                main: vec!["null".to_string()],
                                generics: vec![],
                                nullable: false,
                                kind: dictionary::KindType::Primitive,
                            },
                            line: line.clone(),
                            file: Some(fun.file.clone()),
                        },
                        kind,
                        TypeComparison::NotEqual,
                        line.clone(),
                    ));
                }
                expr_code.push(Instructions::Return);
                merge_code(&mut code.code, &expr_code.code, 0);
                return Ok((max_scope_len, ScopeTerminator::Return));
            }
            crate::codeblock_parser::Nodes::Expr { expr, line } => {
                expression(
                    objects,
                    expr,
                    other_scopes,
                    code,
                    context,
                    &fun,
                    &mut max_scope_len,
                    None,
                    *line,
                )?;
            }
            crate::codeblock_parser::Nodes::Block { body, line } => {
                let scope = open_scope!(body, code);
                if scope.1 != ScopeTerminator::None {
                    return Ok(scope);
                }
            }
            crate::codeblock_parser::Nodes::Break { line, ident } => {
                todo!()
            }
            crate::codeblock_parser::Nodes::Continue { line, ident } => todo!(),
            crate::codeblock_parser::Nodes::Loop { body, line, ident } => {
                use Instructions::*;
                let mut temp_code = Code { code: Vec::new() };
                let scope = open_scope!(body, &mut temp_code);
                temp_code.push(Goto(0));
                merge_code(&mut code.code, &temp_code.code, scope.0);
                if scope.1 != ScopeTerminator::None {
                    return Ok((max_scope_len, ScopeTerminator::Return));
                }
            }
            crate::codeblock_parser::Nodes::Yeet { expr, line } => todo!(),
            crate::codeblock_parser::Nodes::Try {
                body,
                catch,
                finally,
                line,
            } => todo!(),
            crate::codeblock_parser::Nodes::Switch {
                expr,
                body,
                default,
                line,
            } => {
                todo!("stack offset is not correct, this needs fix");
                use Instructions::*;
                let mut expr_code = Code { code: Vec::new() };
                let mut all_return = true;
                expression(
                    objects,
                    expr,
                    other_scopes,
                    &mut expr_code,
                    context,
                    &fun,
                    &mut max_scope_len,
                    None,
                    *line,
                )?;
                // save expr to stack
                let pos = create_var_pos(&other_scopes);
                let cache = last!(other_scopes);
                cache.variables.insert(
                    max_scope_len.to_string(),
                    Variable {
                        kind: None,
                        pos: pos.clone(),
                        value: None,
                        line: line.clone(),
                    },
                );
                max_scope_len += 1;
                expr_code.write(GENERAL_REG1, &pos);
                if body.len() == 0 {
                    match default {
                        Some(default) => {
                            let mut block_code = Code { code: Vec::new() };
                            let (scope, terminator) = open_scope!(default, &mut block_code);
                            merge_code(&mut code.code, &expr_code.code, 0);
                            merge_code(&mut code.code, &block_code.code, 0);
                            return Ok((max_scope_len, terminator));
                        }
                        None => Err(CodegenError::SwitchWithoutCases(line.clone()))?,
                    }
                }
                // after this point we know that there are cases
                // body
                let mut cases = Vec::new();
                for case in body {
                    let mut expr_code = Code { code: Vec::new() };
                    let mut block_code = Code { code: Vec::new() };
                    expression(
                        objects,
                        &case.0,
                        other_scopes,
                        &mut expr_code,
                        context,
                        &fun,
                        &mut max_scope_len,
                        None,
                        *line,
                    )?;
                    let (scope, terminator) = open_scope!(&case.1, &mut block_code);
                    if terminator != ScopeTerminator::Return {
                        all_return = false;
                    }
                    // compare
                    expr_code.read(&pos, GENERAL_REG2);
                    expr_code.extend(&[
                        Equ(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1),
                        Branch(
                            expr_code.code.len() + 2, // +2 because of the branch itself
                            expr_code.code.len() + block_code.code.len() + 3, // +3 because of the branch itself and the goto at the end
                        ),
                    ]);
                    block_code.push(End); // placeholder for goto
                    cases.push((expr_code, block_code, scope));
                }
                let largest_scope = *cases.iter().map(|(_, _, scope)| scope).max().unwrap();
                let mut total_len = cases
                    .iter()
                    .map(|(a, b, _)| a.code.len() + b.code.len())
                    .sum::<usize>()
                    + expr_code.code.len();
                // default
                let defualt = if let Some(default) = default {
                    let mut block_code = Code { code: Vec::new() };
                    let (scope, terminator) = open_scope!(default, &mut block_code);
                    if terminator != ScopeTerminator::Return {
                        all_return = false;
                    }
                    total_len += block_code.code.len();
                    block_code
                } else {
                    Code { code: Vec::new() }
                };
                // merge
                let mut buffer = Vec::new();
                for (expr_code, block_code, scope) in cases.iter_mut() {
                    block_code.push(Goto(total_len + 1)); // +1 because of the goto itself
                    merge_code(&mut buffer, &expr_code.code, *scope);
                    merge_code(&mut buffer, &block_code.code, *scope);
                }
                merge_code(&mut buffer, &defualt.code, largest_scope);
                merge_code(&mut code.code, &buffer, 0);
                if all_return {
                    return Ok((max_scope_len, ScopeTerminator::Return));
                }
                return Ok((max_scope_len, ScopeTerminator::None));
            }
            crate::codeblock_parser::Nodes::Set {
                target,
                expr,
                op,
                line,
            } => {
                use Instructions::*;
                let mut expr_code = Code { code: Vec::new() };
                let mut target_code = Code { code: Vec::new() };
                let mut conclusion_code = Code { code: Vec::new() };
                match target {
                    ValueType::Value(val) => {
                        let pos = gen_value(
                            objects,
                            val,
                            context,
                            other_scopes,
                            &mut target_code,
                            fun,
                            &mut max_scope_len,
                            ExpectedValueType::Pointer,
                            None,
                        )?;
                        match pos {
                            Position::Variable(var, _) => {
                                let var = match find_var(other_scopes, &var) {
                                    Some(var) => var.clone(),
                                    None => Err(CodegenError::VariableNotFound(
                                        var.clone(),
                                        val.root.1.clone(),
                                    ))?,
                                };
                                let expr = expression(
                                    objects,
                                    expr,
                                    other_scopes,
                                    &mut expr_code,
                                    context,
                                    &fun,
                                    &mut max_scope_len,
                                    var.kind.clone(),
                                    *line,
                                )?;
                                if let Operators::Equal = op {
                                    conclusion_code.write(GENERAL_REG1, &var.pos);
                                } else {
                                    conclusion_code.read(&var.pos, GENERAL_REG2);
                                    conclusion_code.push(Swap(GENERAL_REG1, GENERAL_REG2));
                                    match native_operand(
                                        objects,
                                        &op,
                                        &var.kind.clone().unwrap(),
                                        &expr,
                                        &mut conclusion_code,
                                        context,
                                        &fun,
                                        &line,
                                    ) {
                                        Some(_) => (),
                                        None => {
                                            unreachable!("non native operand, this is a bug in the compiler, please report it");
                                        }
                                    };
                                    conclusion_code.write(GENERAL_REG1, &var.pos);
                                }
                                merge_code(&mut code.code, &target_code.code, 0);
                                merge_code(&mut code.code, &expr_code.code, 0);
                                merge_code(&mut code.code, &conclusion_code.code, 0);
                            }
                            Position::Pointer(kind) => {
                                // save pointer to temp var
                                let temp_var = create_var_pos(other_scopes);
                                let cache = last!(other_scopes);
                                cache.variables.insert(
                                    max_scope_len.to_string(),
                                    Variable {
                                        kind: Some(kind.clone()),
                                        pos: temp_var.clone(),
                                        value: None,
                                        line: val.root.1.clone(),
                                    },
                                );
                                max_scope_len += 1;
                                target_code.write(POINTER_REG, &temp_var);
                                let expr = expression(
                                    objects,
                                    expr,
                                    other_scopes,
                                    &mut expr_code,
                                    context,
                                    &fun,
                                    &mut max_scope_len,
                                    Some(kind.clone()),
                                    *line,
                                )?;
                                if let Operators::Equal = op {
                                    conclusion_code.read(&temp_var, POINTER_REG);
                                    conclusion_code.extend(&[WritePtr(GENERAL_REG1)]);
                                } else {
                                    conclusion_code.read(&temp_var, POINTER_REG);
                                    conclusion_code.push(ReadPtr(GENERAL_REG2));
                                    conclusion_code.push(Swap(GENERAL_REG1, GENERAL_REG2));
                                    match native_operand(
                                        objects,
                                        &op,
                                        &kind.clone(),
                                        &expr,
                                        &mut conclusion_code,
                                        context,
                                        &fun,
                                        &line,
                                    ) {
                                        Some(_) => (),
                                        None => {
                                            unreachable!("non native operand, this is a bug in the compiler, please report it");
                                        }
                                    };
                                    conclusion_code.read(&temp_var, POINTER_REG);
                                    conclusion_code.extend(&[WritePtr(GENERAL_REG1)]);
                                }

                                merge_code(&mut code.code, &target_code.code, 0);
                                merge_code(&mut code.code, &expr_code.code, 0);
                                merge_code(&mut code.code, &conclusion_code.code, 0);
                            }
                            _ => todo!(),
                        }
                    }
                    _ => {
                        unreachable!(
                            "target not handled properly by the compiler, please report this bug"
                        )
                    }
                }
            }
        }
    }
    Ok((max_scope_len, ScopeTerminator::None))
}

fn create_var_pos(scopes: &Vec<ScopeCached>) -> MemoryTypes {
    let len = {
        let mut len = 0;
        for scope in scopes {
            len += scope.variables.len();
        }
        len
    };
    MemoryTypes::Stack(len + 0)
}

/// TODO: todo xd
fn try_get_const_val() -> Option<ConstValue> {
    None
}

/// returns starting point and length of appended code
fn merge_code(
    buffer: &mut Vec<Instructions>,
    new_code: &Vec<Instructions>,
    _scope_len: usize,
) -> (usize, usize) {
    let start = buffer.len();
    buffer.reserve(new_code.len());
    let instrs = buffer.len();
    for instr in new_code {
        use Instructions::*;
        let instr = match instr {
            Goto(idx) => {
                let idx = *idx + instrs;
                Goto(idx)
            }
            Branch(pos1, pos2) => {
                let pos1 = *pos1 + instrs;
                let pos2 = *pos2 + instrs;
                Branch(pos1, pos2)
            }
            _ => *instr,
        };
        buffer.push(instr);
    }
    (start, buffer.len())
}

fn flip_stack_access(len: usize, code: &mut Code) {
    use Instructions::*;
    for instr in code.code.iter_mut() {
        match instr {
            Read(from, _) => {
                *from = len - *from + 0;
            }
            Write(to, _) => {
                *to = len - *to + 0;
            }
            Ptr(pos) => {
                *pos = len - *pos + 0;
            }
            _ => (),
        }
    }
}

/// returns location of new constant
fn new_const(
    context: &mut runtime_types::Context,
    value: &intermediate::dictionary::ConstValue,
) -> Result<usize, CodegenError> {
    for (idx, existing) in context.memory.stack.data.iter().enumerate() {
        if value.vm_partial_eq(existing) {
            return Ok(idx);
        }
    }
    let idx = context.memory.stack.data.len();
    match value {
        intermediate::dictionary::ConstValue::Function(_) => todo!(),
        intermediate::dictionary::ConstValue::String(str) => {
            let pos = context.memory.strings.from_str(str);
            context
                .memory
                .stack
                .data
                .push(runtime::runtime_types::Types::Pointer(
                    pos,
                    runtime::runtime_types::PointerTypes::String,
                ));
        }
        intermediate::dictionary::ConstValue::Array(arr) => {
            let size = arr.len();
            let pos = context.memory.allocate_obj(size + 1);
            context
                .memory
                .stack
                .data
                .push(runtime::runtime_types::Types::Pointer(
                    pos,
                    runtime::runtime_types::PointerTypes::Object,
                ));
            for (idx, value) in arr.iter().enumerate() {
                let idx = idx + 1;
                let pos = context.memory.allocate_obj(1);
                context.memory.heap.data[pos][idx] = value.to_runtime();
            }
            let np_pos = context.memory.non_primitives.len();
            context
                .memory
                .non_primitives
                .push(runtime_types::NonPrimitiveType {
                    name: format!("[{}]", size),
                    kind: runtime_types::NonPrimitiveTypes::Array,
                    len: size,
                    pointers: todo!(),
                    methods: HashMap::new(), // todo: implement iterator trait, etc.
                });
            context.memory.heap.data[pos][0] = runtime::runtime_types::Types::NonPrimitive(todo!())
        }
        intermediate::dictionary::ConstValue::Undefined => {
            Err(CodegenError::CannotInitializeConstant)?
        }
        _ => {
            context.memory.stack.data.push(value.to_runtime());
        }
    }
    Ok(idx)
}

#[derive(Debug, Clone)]
struct ScopeCached {
    variables: HashMap<String, Variable>,
}
#[derive(Debug, Clone)]
struct Variable {
    kind: Option<Kind>,
    pos: MemoryTypes,
    value: Option<ConstValue>,
    line: Line,
}

struct Code {
    pub code: Vec<Instructions>,
}

impl Code {
    pub fn read(&mut self, from: &MemoryTypes, to: usize) {
        match from {
            MemoryTypes::Stack(n) => {
                self.code.push(Instructions::Read(*n, to));
            }
            MemoryTypes::Register(n) => {
                let n = n.to_num();
                match n == to {
                    true => {}
                    false => self.code.push(Instructions::Move(n, to)),
                }
            }
        }
    }
    pub fn ptr(&mut self, from: &MemoryTypes, to: usize) {
        match from {
            MemoryTypes::Stack(n) => {
                self.code.push(Instructions::Ptr(*n));
                if to != GENERAL_REG1 {
                    self.code.push(Instructions::Move(GENERAL_REG1, to));
                }
            }
            MemoryTypes::Register(n) => {
                let n = n.to_num();
                match n == to {
                    true => {}
                    false => self.code.push(Instructions::Move(n, to)),
                }
            }
        }
    }
    pub fn write(&mut self, from: usize, to: &MemoryTypes) {
        match to {
            MemoryTypes::Stack(n) => self.code.push(Instructions::Write(*n, from)),
            MemoryTypes::Register(n) => {
                let n = n.to_num();
                match n == from {
                    true => {}
                    false => self.code.push(Instructions::Move(from, n)),
                }
            }
        }
    }
    pub fn extend(&mut self, other: &[Instructions]) {
        self.code.extend(other)
    }
    pub fn push(&mut self, instr: Instructions) {
        self.code.push(instr)
    }
}

#[derive(Debug, Clone)]
pub enum CompoundField {
    Field(String),
    Method(String),
    OverloadedOperator(Operators),
    TraitMethod(String, String),
}

impl CompoundField {
    pub fn get_ident(&self) -> String {
        match self {
            CompoundField::Field(ident) => ident.clone(),
            CompoundField::Method(ident) => ident.clone(),
            _ => unreachable!("this is a bug in the compiler, please report it"),
        }
    }
}

#[derive(Debug, Clone)]
enum Position {
    Function(FunctionKind, Kind),
    CompoundField(InnerPath, CompoundField, Kind),
    Import(String),
    BinImport(String),
    Variable(String, Kind),
    Pointer(Kind),
    Compound(Kind),
    Value(Kind),
}

impl Position {
    pub fn get_kind(&self) -> Result<Kind, CodegenError> {
        Ok(match self {
            Position::Function(_, kind) => kind.clone(),
            Position::CompoundField(_, _, kind) => kind.clone(),
            Position::Variable(_, kind) => kind.clone(),
            Position::Pointer(kind) => kind.clone(),
            Position::Compound(kind) => kind.clone(),
            Position::Value(kind) => kind.clone(),
            _ => Err(CodegenError::CannotGetKind(self.clone()))?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InnerPath {
    file: String,
    block: Option<String>,
    ident: String,
    kind: ImportKinds,
}

impl InnerPath {
    pub fn main() -> Self {
        InnerPath {
            file: "main.rd".to_string(),
            block: None,
            ident: "main".to_string(),
            kind: ImportKinds::Rd,
        }
    }
    pub fn get_mut<'a>(&'a self, objects: &'a mut Context) -> Result<&mut Function, CodegenError> {
        let file = match objects.0.get_mut(&self.file) {
            Some(f) => f,
            None => {
                return Err(CodegenError::FunctionNotFound(self.clone()));
            }
        };
        match &self.block {
            Some(b) => {
                for structt in file.structs.iter_mut() {
                    if &structt.identifier == b {
                        for fun in structt.functions.iter_mut() {
                            if fun.identifier.as_ref().unwrap() == &self.ident
                                || fun.identifier.as_ref().unwrap() == b
                            {
                                return Ok(fun);
                            }
                        }
                    }
                }
                Err(CodegenError::FunctionNotFound(self.clone()))
            }
            None => {
                for fun in file.functions.iter_mut() {
                    if fun.identifier.clone().unwrap().as_ref() == self.ident {
                        return Ok(fun);
                    }
                }
                Err(CodegenError::FunctionNotFound(self.clone()))
            }
        }
    }
    pub fn get<'a>(&'a self, objects: &'a Context) -> Result<&Function, CodegenError> {
        let file = match objects.0.get(&self.file) {
            Some(f) => f,
            None => {
                return Err(CodegenError::FunctionNotFound(self.clone()));
            }
        };
        match &self.block {
            Some(b) => {
                for structt in file.structs.iter() {
                    if &structt.identifier == b {
                        for fun in structt.functions.iter() {
                            if &fun.identifier.clone().unwrap() == &self.ident {
                                return Ok(fun);
                            }
                        }
                    }
                }
                Err(CodegenError::FunctionNotFound(self.clone()))
            }
            None => {
                for fun in file.functions.iter() {
                    if fun.identifier.clone().unwrap().as_ref() == self.ident {
                        return Ok(fun);
                    }
                }
                Err(CodegenError::FunctionNotFound(self.clone()))
            }
        }
    }
    pub fn get_bin<'a>(
        &'a self,
        objects: &'a Context,
    ) -> Result<&libloader::Function, CodegenError> {
        let file = match objects.1.get(&self.file) {
            Some(f) => f,
            None => {
                return Err(CodegenError::FunctionNotFound(self.clone()));
            }
        };
        match &self.block {
            Some(b) => {
                for structt in file.structs.iter() {
                    if &structt.name == b {
                        for fun in structt.methods.iter() {
                            if &fun.name.clone() == &self.ident {
                                return Ok(fun);
                            }
                        }
                    }
                }
                for userdata in file.user_data.iter() {
                    if &userdata.name == b {
                        for fun in userdata.methods.iter() {
                            if &fun.name.clone() == &self.ident {
                                return Ok(fun);
                            }
                        }
                    }
                }
                Err(CodegenError::FunctionNotFound(self.clone()))
            }
            None => {
                for fun in file.functions.iter() {
                    if fun.name == self.ident {
                        return Ok(fun);
                    }
                }
                Err(CodegenError::FunctionNotFound(self.clone()))
            }
        }
    }
}

fn native_operand(
    objects: &mut Context,
    op: &tokenizer::Operators,
    left: &Kind,
    right: &Kind,
    code: &mut Code,
    context: &mut runtime_types::Context,
    fun: &InnerPath,
    line: &Line,
) -> Option<Kind> {
    use Instructions::*;
    match op {
        Operators::Plus => {
            if left.is_number() && right.is_number() && left.cmp(right).is_equal() {
                code.extend(&[Add(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1)]);
                return Some(left.clone());
            } else if left.is_number() && right.is_number() && left.cmp(right).is_not_equal() {
                cast(objects, left, right, code, context, fun, line, GENERAL_REG2)?;
                code.extend(&[Add(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1)]);
                return Some(right.clone());
            } else if left.is_string() && right.is_string() {
                code.extend(&[Cal(CORE_LIB, 2), Move(RETURN_REG, GENERAL_REG1)]);
                return Some(Kind {
                    body: TypeBody::Type {
                        refs: 0,
                        main: vec!["string".to_string()],
                        generics: vec![],
                        nullable: false,
                        kind: dictionary::KindType::Primitive,
                    },
                    line: line.clone(),
                    file: Some(fun.file.clone()),
                });
            } else if left.is_string() && right.is_primitive_simple() {
                cast(
                    objects,
                    right,
                    &Kind {
                        body: TypeBody::Type {
                            refs: 0,
                            main: vec!["string".to_string()],
                            generics: vec![],
                            nullable: false,
                            kind: dictionary::KindType::Primitive,
                        },
                        line: line.clone(),
                        file: Some(fun.file.clone()),
                    },
                    code,
                    context,
                    fun,
                    line,
                    GENERAL_REG2,
                )?;
                code.extend(&[Cal(CORE_LIB, 2), Move(RETURN_REG, GENERAL_REG1)]);
                return Some(Kind {
                    body: TypeBody::Type {
                        refs: 0,
                        main: vec!["string".to_string()],
                        generics: vec![],
                        nullable: false,
                        kind: dictionary::KindType::Primitive,
                    },
                    line: line.clone(),
                    file: Some(fun.file.clone()),
                });
            } else if left.is_primitive_simple() && right.is_string() {
                code.push(Swap(GENERAL_REG1, GENERAL_REG2));
                cast(
                    objects,
                    left,
                    &Kind {
                        body: TypeBody::Type {
                            refs: 0,
                            main: vec!["string".to_string()],
                            generics: vec![],
                            nullable: false,
                            kind: dictionary::KindType::Primitive,
                        },
                        line: line.clone(),
                        file: Some(fun.file.clone()),
                    },
                    code,
                    context,
                    fun,
                    line,
                    GENERAL_REG2,
                )?;
                code.extend(&[
                    Swap(GENERAL_REG1, GENERAL_REG2),
                    Cal(CORE_LIB, 2),
                    Move(RETURN_REG, GENERAL_REG1),
                ]);
                return Some(Kind {
                    body: TypeBody::Type {
                        refs: 0,
                        main: vec!["string".to_string()],
                        generics: vec![],
                        nullable: false,
                        kind: dictionary::KindType::Primitive,
                    },
                    line: line.clone(),
                    file: Some(fun.file.clone()),
                });
            } else {
                None?
            }
        }
        Operators::Minus => {
            if left.is_number() && right.is_number() && left.cmp(right).is_equal() {
                code.extend(&[Sub(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1)]);
                return Some(left.clone());
            } else {
                None?
            }
        }
        Operators::Star => {
            if left.is_number() && right.is_number() && left.cmp(right).is_equal() {
                code.extend(&[Mul(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1)]);
                return Some(left.clone());
            } else {
                None?
            }
        }
        Operators::Slash => {
            if left.is_number() && right.is_number() && left.cmp(right).is_equal() {
                code.extend(&[Div(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1)]);
                return Some(left.clone());
            } else {
                None?
            }
        }
        Operators::Mod => {
            if left.is_number() && right.is_number() && left.cmp(right).is_equal() {
                code.extend(&[Mod(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1)]);
                return Some(left.clone());
            } else {
                None?
            }
        }
        Operators::Equal => {
            if left.is_primitive_simple()
                && right.is_primitive_simple()
                && left.cmp(right).is_equal()
            {
                code.extend(&[Equ(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1)]);
                return Some(Kind {
                    body: TypeBody::Type {
                        refs: 0,
                        main: vec!["bool".to_string()],
                        generics: vec![],
                        nullable: false,
                        kind: dictionary::KindType::Primitive,
                    },
                    line: line.clone(),
                    file: Some(fun.file.clone()),
                });
            } else {
                None?
            }
        }
        Operators::AddEq => {
            if left.is_number() && right.is_number() && left.cmp(right).is_equal() {
                code.extend(&[Add(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1)]);
                return Some(left.clone());
            } else {
                None?
            }
        }
        Operators::SubEq => {
            if left.is_number() && right.is_number() && left.cmp(right).is_equal() {
                code.extend(&[Sub(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1)]);
                return Some(left.clone());
            } else {
                None?
            }
        }
        Operators::MulEq => {
            if left.is_number() && right.is_number() && left.cmp(right).is_equal() {
                code.extend(&[Mul(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1)]);
                return Some(left.clone());
            } else {
                None?
            }
        }
        Operators::DivEq => {
            if left.is_number() && right.is_number() && left.cmp(right).is_equal() {
                code.extend(&[Div(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1)]);
                return Some(left.clone());
            } else {
                None?
            }
        }
        Operators::DoubleEq => {
            if left.is_string() && right.is_string() {
                code.extend(&[Cal(CORE_LIB, 3), Move(RETURN_REG, GENERAL_REG1)]);
                return Some(Kind {
                    body: TypeBody::Type {
                        refs: 0,
                        main: vec!["bool".to_string()],
                        generics: vec![],
                        nullable: false,
                        kind: dictionary::KindType::Primitive,
                    },
                    line: line.clone(),
                    file: Some(fun.file.clone()),
                });
            }
            if left.is_primitive_simple()
                && right.is_primitive_simple()
                && left.cmp(right).is_equal()
            {
                code.extend(&[Equ(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1)]);
                return Some(Kind {
                    body: TypeBody::Type {
                        refs: 0,
                        main: vec!["bool".to_string()],
                        generics: vec![],
                        nullable: false,
                        kind: dictionary::KindType::Primitive,
                    },
                    line: line.clone(),
                    file: Some(fun.file.clone()),
                });
            } else {
                None?
            }
        }
        Operators::NotEqual => {
            if left.is_null() && right.is_null() {
                code.extend(&[Equ(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1)]);
                return Some(Kind {
                    body: TypeBody::Type {
                        refs: 0,
                        main: vec!["bool".to_string()],
                        generics: vec![],
                        nullable: false,
                        kind: dictionary::KindType::Primitive,
                    },
                    line: line.clone(),
                    file: Some(fun.file.clone()),
                });
            }
            if left.is_string() && right.is_string() {
                code.extend(&[
                    Cal(CORE_LIB, 3),
                    Move(RETURN_REG, GENERAL_REG1),
                    Not(GENERAL_REG1, GENERAL_REG1),
                ]);
                return Some(Kind {
                    body: TypeBody::Type {
                        refs: 0,
                        main: vec!["bool".to_string()],
                        generics: vec![],
                        nullable: false,
                        kind: dictionary::KindType::Primitive,
                    },
                    line: line.clone(),
                    file: Some(fun.file.clone()),
                });
            }
            if left.is_primitive_simple()
                && right.is_primitive_simple()
                && left.cmp(right).is_equal()
            {
                code.extend(&[
                    Equ(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1),
                    Not(GENERAL_REG1, GENERAL_REG1),
                ]);
                return Some(Kind {
                    body: TypeBody::Type {
                        refs: 0,
                        main: vec!["bool".to_string()],
                        generics: vec![],
                        nullable: false,
                        kind: dictionary::KindType::Primitive,
                    },
                    line: line.clone(),
                    file: Some(fun.file.clone()),
                });
            } else {
                None?
            }
        }
        Operators::And => {
            if left.is_primitive_simple()
                && right.is_primitive_simple()
                && left.cmp(right).is_equal()
            {
                code.extend(&[And(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1)]);
                return Some(Kind {
                    body: TypeBody::Type {
                        refs: 0,
                        main: vec!["bool".to_string()],
                        generics: vec![],
                        nullable: false,
                        kind: dictionary::KindType::Primitive,
                    },
                    line: line.clone(),
                    file: Some(fun.file.clone()),
                });
            } else {
                None?
            }
        }
        Operators::Or => {
            if left.is_primitive_simple()
                && right.is_primitive_simple()
                && left.cmp(right).is_equal()
            {
                code.extend(&[Or(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1)]);
                return Some(Kind {
                    body: TypeBody::Type {
                        refs: 0,
                        main: vec!["bool".to_string()],
                        generics: vec![],
                        nullable: false,
                        kind: dictionary::KindType::Primitive,
                    },
                    line: line.clone(),
                    file: Some(fun.file.clone()),
                });
            } else {
                None?
            }
        }
        Operators::Ampersant => {
            if left.is_primitive_simple()
                && right.is_primitive_simple()
                && left.cmp(right).is_equal()
            {
                code.extend(&[And(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1)]);
                return Some(Kind {
                    body: TypeBody::Type {
                        refs: 0,
                        main: vec!["bool".to_string()],
                        generics: vec![],
                        nullable: false,
                        kind: dictionary::KindType::Primitive,
                    },
                    line: line.clone(),
                    file: Some(fun.file.clone()),
                });
            } else {
                None?
            }
        }
        Operators::Pipe => {
            if left.is_primitive_simple()
                && right.is_primitive_simple()
                && left.cmp(right).is_equal()
            {
                code.extend(&[Or(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1)]);
                return Some(Kind {
                    body: TypeBody::Type {
                        refs: 0,
                        main: vec!["bool".to_string()],
                        generics: vec![],
                        nullable: false,
                        kind: dictionary::KindType::Primitive,
                    },
                    line: line.clone(),
                    file: Some(fun.file.clone()),
                });
            } else {
                None?
            }
        }
        Operators::AngleBracket(side) => match side {
            false => {
                if left.is_number() && right.is_number() && left.cmp(right).is_equal() {
                    code.extend(&[Less(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1)]);
                    return Some(Kind {
                        body: TypeBody::Type {
                            refs: 0,
                            main: vec!["bool".to_string()],
                            generics: vec![],
                            nullable: false,
                            kind: dictionary::KindType::Primitive,
                        },
                        line: line.clone(),
                        file: Some(fun.file.clone()),
                    });
                } else {
                    None?
                }
            }
            true => {
                if left.is_number() && right.is_number() && left.cmp(right).is_equal() {
                    code.extend(&[Grt(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1)]);
                    return Some(Kind {
                        body: TypeBody::Type {
                            refs: 0,
                            main: vec!["bool".to_string()],
                            generics: vec![],
                            nullable: false,
                            kind: dictionary::KindType::Primitive,
                        },
                        line: line.clone(),
                        file: Some(fun.file.clone()),
                    });
                } else {
                    None?
                }
            }
        },
        Operators::LessEq => {
            if left.is_number() && right.is_number() && left.cmp(right).is_equal() {
                code.extend(&[
                    Grt(GENERAL_REG2, GENERAL_REG1, GENERAL_REG1),
                    Not(GENERAL_REG1, GENERAL_REG1),
                ]);
                return Some(Kind {
                    body: TypeBody::Type {
                        refs: 0,
                        main: vec!["bool".to_string()],
                        generics: vec![],
                        nullable: false,
                        kind: dictionary::KindType::Primitive,
                    },
                    line: line.clone(),
                    file: Some(fun.file.clone()),
                });
            } else {
                None?
            }
        }
        Operators::MoreEq => {
            if left.is_number() && right.is_number() && left.cmp(right).is_equal() {
                code.extend(&[
                    Less(GENERAL_REG2, GENERAL_REG1, GENERAL_REG1),
                    Not(GENERAL_REG1, GENERAL_REG1),
                ]);
                return Some(Kind {
                    body: TypeBody::Type {
                        refs: 0,
                        main: vec!["bool".to_string()],
                        generics: vec![],
                        nullable: false,
                        kind: dictionary::KindType::Primitive,
                    },
                    line: line.clone(),
                    file: Some(fun.file.clone()),
                });
            } else {
                None?
            }
        }
        Operators::Not => unreachable!("'not' is not a binary operator, this is a bug in compiler"),
    }
    None
}

fn native_unary_operand(
    objects: &mut Context,
    op: &Option<(tokenizer::Operators, Line)>,
    kind: &Kind,
    code: &mut Code,
    context: &mut runtime_types::Context,
    fun: &InnerPath,
    line: &Line,
    register: usize,
) -> Result<Kind, CodegenError> {
    use Instructions::*;
    if op.is_none() {
        return Ok(kind.clone());
    }
    match op.unwrap().0 {
        Operators::Not => {
            if kind.is_primitive_simple() {
                if cast(
                    objects,
                    kind,
                    &Kind {
                        body: TypeBody::Type {
                            refs: 0,
                            main: vec!["bool".to_string()],
                            generics: vec![],
                            nullable: false,
                            kind: dictionary::KindType::Primitive,
                        },
                        line: line.clone(),
                        file: Some(fun.file.clone()),
                    },
                    code,
                    context,
                    fun,
                    line,
                    register,
                )
                .is_none()
                {
                    return Err(CodegenError::CouldNotCastTo(
                        kind.clone(),
                        Kind {
                            body: TypeBody::Type {
                                refs: 0,
                                main: vec!["bool".to_string()],
                                generics: vec![],
                                nullable: false,
                                kind: dictionary::KindType::Primitive,
                            },
                            line: line.clone(),
                            file: Some(fun.file.clone()),
                        },
                        line.clone(),
                    ));
                }
                code.extend(&[Not(register, register)]);
                return Ok(Kind {
                    body: TypeBody::Type {
                        refs: 0,
                        main: vec!["bool".to_string()],
                        generics: vec![],
                        nullable: false,
                        kind: dictionary::KindType::Primitive,
                    },
                    line: line.clone(),
                    file: Some(fun.file.clone()),
                });
            } else {
                Err(CodegenError::ExpectedBool(line.clone()))?
            }
        }
        Operators::Minus => {
            if kind.is_number() {
                code.extend(&[Neg(register)]);
                return Ok(kind.clone());
            } else {
                Err(CodegenError::ExpectedNumber(line.clone()))?
            }
        }
        _ => Err(CodegenError::UnaryNotApplicable(
            kind.clone(),
            op.unwrap().0,
            op.unwrap().1,
        ))?,
    }
}

fn cast(
    objects: &mut Context,
    from: &Kind,
    to: &Kind,
    code: &mut Code,
    context: &mut runtime_types::Context,
    fun: &InnerPath,
    line: &Line,
    register: usize,
) -> Option<()> {
    use Instructions::*;
    if
    /*to.array_depth > 0 && to.array_depth == from.array_depth*/
    to.is_array() && from.is_array() {
        return Some(());
    }
    if to.get_nullable() && from.is_null() {
        return Some(());
    }
    if to.is_null() && from.get_nullable() {
        return None;
    }
    if from.cmp(to).is_equal() {
        return Some(());
    }
    if from.is_primitive_simple() && to.is_primitive_simple() {
        if to.is_string() {
            if register != GENERAL_REG2 {
                code.extend(&[Move(register, GENERAL_REG2)]);
            }
            code.extend(&[Cal(CORE_LIB, 1), Move(RETURN_REG, register)]);
            return Some(());
        }
        if (from.is_number() || from.is_bool()) && (to.is_number() || to.is_bool()) {
            let const_val = match new_const(context, &to.into_const().expect("Failed to recognize number or bool, this is a bug in the compiler, please report it")) {
                Ok(pos) => pos,
                Err(_) => unreachable!("Failed to process number or bool, this is a bug in the compiler, please report it"),
            };
            code.extend(&[
                ReadConst(const_val, POINTER_REG),
                Cast(register, POINTER_REG),
            ]);
            return Some(());
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ScopeTerminator {
    Return,
    Break(String),
    Continue(String),
    Yeet,
    None,
}

fn correct_kind(
    objects: &Context,
    _kind: &Kind,
    fun: &InnerPath,
    line: &Line,
) -> Result<Kind, CodegenError> {
    if _kind.is_primitive() {
        let mut kind = _kind.clone();
        *kind.kind_mut() = dictionary::KindType::Primitive;
        return Ok(kind);
    }
    match &_kind.body {
        TypeBody::Type {
            refs,
            main,
            generics,
            nullable,
            kind,
        } => {
            let mut file = fun.clone();
            file.file = _kind.file.clone().unwrap_or(file.file);
            for i in 0..main.len() - 1 {
                let import = match find_import(objects, &main[i], &file.file) {
                    Some(import) => import,
                    None => Err(CodegenError::ImportNotFound(main[i].clone(), line.clone()))?,
                };
                file = InnerPath {
                    file: import.0.to_string(),
                    block: None,
                    ident: "".to_string(),
                    kind: import.1,
                };
            }
            file.ident = main.last().unwrap().clone();
            let mut real = get_kind(objects, &file, line)?;
            *real.refs_mut() = *refs;
            *real.nullable_mut() = *nullable;
            real.line = line.clone();
            Ok(real)
        }
        _ => Ok(_kind.clone()),
    }
}

fn get_kind(objects: &Context, location: &InnerPath, line: &Line) -> Result<Kind, CodegenError> {
    if let Some(file) = objects.0.get(&location.file) {
        for fun in file.functions.iter() {
            if fun.identifier.clone().unwrap().as_ref() == location.ident {
                return Ok(Kind::from_fun(fun, location.file.clone()));
            }
        }
        for structt in file.structs.iter() {
            if structt.identifier == location.ident {
                return Ok(Kind::from_struct(
                    structt.identifier.clone(),
                    location.file.clone(),
                    structt.line,
                ));
            }
        }
        for traitt in file.traits.iter() {
            if traitt.identifier == location.ident {
                return Ok(Kind::from_trait(traitt, location.file.clone()));
            }
        }
        for enumm in file.enums.iter() {
            if enumm.identifier == location.ident {
                return Ok(Kind::from_enum(enumm, location.file.clone()));
            }
        }
    return Err(CodegenError::KindNotFound(
            location.ident.clone(),
            line.clone(),
        ));
    }
    let file = match objects.1.get(&location.file) {
        Some(f) => f,
        None => {
    return Err(CodegenError::KindNotFound(
                location.ident.clone(),
                line.clone(),
            ));
        }
    };
    for fun in file.functions.iter() {
        if fun.name == location.ident {
            return Ok(Kind::bfrom_fun(fun, location.file.clone(), line.clone()));
        }
    }
    for user_data in file.user_data.iter() {
        if user_data.name == location.ident {
            return Ok(Kind::from_userdata(
                user_data.name.clone(),
                location.file.clone(),
                user_data.line,
            ));
        }
    }
    Err(CodegenError::KindNotFound(
        location.ident.clone(),
        line.clone(),
    ))
}

pub enum ExpectedValueType {
    Value,
    Pointer,
}
