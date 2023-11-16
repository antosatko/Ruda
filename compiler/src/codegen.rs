use std::collections::HashMap;
use std::fmt::Pointer;
use intermediate::dictionary::ImportKinds;

use runtime::runtime_types::{
    self, Instructions, Memory, Stack, Types, CODE_PTR_REG, GENERAL_REG1, GENERAL_REG3,
    MEMORY_REG1, MEMORY_REG3, POINTER_REG, RETURN_REG,
};

use crate::codeblock_parser::Nodes;
use crate::expression_parser::{self, FunctionCall, ValueType};
use crate::intermediate::dictionary::{
    self, Arg, ConstValue, Function, ShTypeBuilder, ShallowType, TypeComparison,
};
use crate::lexer::tokenizer::{Operators, Tokens};
use crate::tree_walker::tree_walker::{Err, Line};
use crate::{intermediate, prep_objects::Context};

use crate::libloader::{MemoryTypes, Registers, self};

pub fn gen(
    objects: &mut Context,
    main: &str,
) -> Result<runtime::runtime_types::Context, CodegenError> {
    let mut vm_context = runtime::runtime_types::Context::new();
    // Initialize some common constants for faster lookup
    let consts = [
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
    gen_fun(objects, &main_path, &mut vm_context)?;
    call_main(main_path.get(objects)?, &mut vm_context)?;
    Ok(vm_context)
}

fn call_main(main: &Function, context: &mut runtime_types::Context) -> Result<(), CodegenError> {
    use Instructions::*;
    context.code.entry_point = context.code.data.len() + 1;
    let consts_len = context.memory.stack.data.len();
    context.code.data.extend(&[
        End,
        ReserveStack(consts_len, 0),
        ReserveStack(main.stack_size.unwrap(), main.pointers.unwrap_or(0)),
        Goto(main.location.unwrap()),
    ]);
    // swap all returns in main with end
    for i in main.location.unwrap()..context.code.data.len() {
        match context.code.data[i] {
            Return => {
                context.code.data[i] = End;
            }
            _ => {}
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
    VariableTypeMismatch(ShallowType, ShallowType, TypeComparison, Line),
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
    ArgTypeMismatch(Arg, ShallowType, TypeComparison, Line),
    CannotAttachMethodsToFunctions(Line),
    ImportIsNotAValidValue(Line),
    IncorrectArgs(Line),
}

pub fn stringify(
    context: &runtime::runtime_types::Context,
    shlibs: &Vec<stringify::ShLib>,
) -> String {
    use stringify::stringify;
    stringify(context, Some(&shlibs))
}

fn gen_fun<'a>(
    objects: &'a mut Context,
    fun: &'a InnerPath,
    context: &'a mut runtime_types::Context,
) -> Result<(), CodegenError> {
    let mut code = Code { code: Vec::new() };
    let mut args_scope_len = 0;
    let mut scopes = vec![ScopeCached {
        variables: HashMap::new(),
    }];
    let this_fun = fun.get(objects)?;
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
    let scope_len = get_scope(
        objects,
        &this_fun.code.clone(),
        context,
        &mut scopes,
        &mut code,
        fun,
    )? + args_scope_len;
    println!("scope_len: {}", scope_len);
    println!("variables: {:#?}", scopes);
    flip_stack_access(scope_len, &mut code);
    code.push(Instructions::Return);
    let mut args_code = Code{ code: Vec::new() };
    for (idx, arg) in scopes[0].variables.iter().enumerate() {
        args_code.extend(&[
            Instructions::Move(MEMORY_REG3, POINTER_REG),
            Instructions::IndexStatic(idx),
            Instructions::ReadPtr(GENERAL_REG1),
            Instructions::Write(scope_len-idx, GENERAL_REG1)
        ]);
    }
    let mut temp = Vec::new();
    merge_code(&mut temp, &args_code.code, scope_len);
    merge_code(&mut temp, &code.code, scope_len);
    let pos = merge_code(&mut context.code.data, &temp, scope_len);
    let this_fun = fun.get_mut(objects)?;
    this_fun.location = Some(pos.0);
    this_fun.stack_size = Some(scope_len);
    Ok(())
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
) -> Result<ShallowType, CodegenError> {
    use Instructions::*;
    let mut return_kind = ShallowType::empty();
    match expr {
        ValueType::Literal(lit) => {
            match &lit.value {
                expression_parser::Literals::Number(tok) => {
                    let const_num = match tok.into_const_number() {
                        Some(num) => num,
                        None => {
                            unreachable!("number not handled properly by the compiler, please report this bug");
                        }
                    };
                    let pos = new_const(context, &const_num.0)?;
                    code.push(ReadConst(pos, GENERAL_REG1));
                    return_kind = const_num.1;
                }
                expression_parser::Literals::Array(arr) => {
                    let pos = {
                        let mut arr_ = Vec::new();
                        match arr {
                            expression_parser::ArrayRule::Fill { value, size } => todo!(),
                            expression_parser::ArrayRule::Explicit(values) => {
                                code.extend(&[
                                    AllocateStatic(values.len() + 1),
                                    Move(GENERAL_REG1, POINTER_REG),
                                ]);
                                for value in values {
                                    arr_.push(expression(
                                        objects, value, scopes, code, context, &fun, scope_len,
                                    )?);
                                }
                            }
                        }
                    };
                }
                expression_parser::Literals::String(str) => {
                    let pos = new_const(context, &ConstValue::String(str.clone()))?;
                    match lit.refs {
                        expression_parser::Ref::Dereferencing(depth) => {
                            Err(CodegenError::DerefereString(depth, lit.line.clone()))?
                        }
                        expression_parser::Ref::Reference(depth) => {
                            if depth > 1 {
                                Err(CodegenError::ReferenceString(depth, lit.line.clone()))?;
                            }
                            code.extend(&[ReadConst(pos, GENERAL_REG1), Debug(GENERAL_REG1)]);
                        }
                        expression_parser::Ref::None => {
                            code.extend(&[
                                ReadConst(pos, POINTER_REG),
                                Cal(1, 3),
                                Move(RETURN_REG, GENERAL_REG1),
                                Debug(GENERAL_REG1),
                            ]);
                        }
                    }
                }
                expression_parser::Literals::Char(c) => {
                    let pos = new_const(context, &ConstValue::Char(*c))?;
                    code.push(ReadConst(pos, GENERAL_REG1));
                }
            }
        }
        ValueType::AnonymousFunction(_) => todo!(),
        ValueType::Parenthesis(expr, tail) => {
            let kind = expression(objects, expr, scopes, code, context, &fun, scope_len)?;
            for (node, line) in tail.iter() {
                match node {
                    expression_parser::TailNodes::Nested(_) => todo!(),
                    expression_parser::TailNodes::Index(_) => todo!(),
                    expression_parser::TailNodes::Call(_) => todo!(),
                    expression_parser::TailNodes::Cast(_) => todo!(),
                }
            }
            return_kind = kind;
        }
        ValueType::Expression(_) => todo!(),
        ValueType::Operator(_, line) => unreachable!(
            "operator not handled properly by the compiler at {line}, please report this bug"
        ),
        ValueType::Value(value) => {
            // check for inner const
            if value.is_true_simple() && value.root.0 == "true" {
                code.push(ReadConst(1, GENERAL_REG1));
                return Ok(ShTypeBuilder::new().set_name("bool").build());
            } else if value.is_true_simple() && value.root.0 == "false" {
                code.push(ReadConst(2, GENERAL_REG1));
                return Ok(ShTypeBuilder::new().set_name("bool").build());
            } else if value.is_true_simple() && value.root.0 == "null" {
                code.push(ReadConst(0, GENERAL_REG1));
                return Ok(ShTypeBuilder::new().set_name("null").build());
            }
            return gen_value(objects, value, context, scopes, code, fun, scope_len);
        }
        ValueType::Blank => {}
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

fn find_import<'a>(objects: &'a Context, ident: &'a str, file_name: &'a str) -> Option<(&'a str, intermediate::dictionary::ImportKinds)> {
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
                        kind: ImportKinds::Rd
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
                        kind: ImportKinds::Dll
                    }));
                }
            }
        }
        None => None?,
    };
    None
}

#[derive(Debug, Clone)]
enum FunctionKind {
    Fun(InnerPath),
    Binary(InnerPath),
    Dynamic(ShallowType),
}

fn gen_value(
    objects: &mut Context,
    value: &expression_parser::Variable,
    context: &mut runtime_types::Context,
    scopes: &mut Vec<ScopeCached>,
    code: &mut Code,
    fun: &InnerPath,
    scope_len: &mut usize,
) -> Result<ShallowType, CodegenError> {
    use Instructions::*;
    let root = identify_root(
        objects,
        &value.root.0,
        Some(scopes),
        &fun.file,
        &value.root.1,
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
    let kind = match pos {
        Position::Function(fun) => match fun {
            FunctionKind::Fun(fun) => {
                let fun = fun.get(objects)?;
                fun.return_type.clone().unwrap()
            }
            _ => todo!(),
        },
        Position::StructField(_, _) => todo!(),
        Position::Import(_) => Err(CodegenError::ImportIsNotAValidValue(value.root.1.clone()))?,
        Position::BinImport(_) => Err(CodegenError::ImportIsNotAValidValue(value.root.1.clone()))?,
        Position::Variable(var) => {
            let var = match find_var(&scopes, &var) {
                Some(var) => var,
                None => Err(CodegenError::VariableNotFound(
                    var.clone(),
                    value.root.1.clone(),
                ))?,
            };
            let pos_cloned = var.pos.clone();
            code.read(&pos_cloned, GENERAL_REG1);
            var.kind.clone().unwrap()
        }
        Position::Pointer(kind) => kind,
        Position::ReturnValue(kind) => kind,
    };
    Ok(kind)
}

fn identify_root(
    objects: &Context,
    ident: &str,
    scopes: Option<&mut Vec<ScopeCached>>,
    file: &str,
    line: &Line,
) -> Result<Position, CodegenError> {
    if let Some(scopes) = scopes {
        if find_var(scopes, &ident).is_some() {
            return Ok(Position::Variable(ident.to_string()));
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
        return Ok(Position::Function(fun));
    }
    // THIS ONE THIS ONE THIS ONE THIS ONE THIS ONE THIS ONE THIS ONE THIS ONE 
    // THIS ONE THIS ONE THIS ONE THIS ONE THIS ONE THIS ONE THIS ONE THIS ONE 
    // THIS ONE THIS ONE THIS ONE THIS ONE THIS ONE THIS ONE THIS ONE THIS ONE 
    Err(CodegenError::VariableNotFound(
        ident.to_string(),
        line.clone(),
    ))?
    // THIS ONE THIS ONE THIS ONE THIS ONE THIS ONE THIS ONE THIS ONE THIS ONE 
    // THIS ONE THIS ONE THIS ONE THIS ONE THIS ONE THIS ONE THIS ONE THIS ONE 
    // THIS ONE THIS ONE THIS ONE THIS ONE THIS ONE THIS ONE THIS ONE THIS ONE 
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
    let mut return_kind = ShallowType::empty();
    match tail.next() {
        Some(node) => {
            match &node.0 {
                expression_parser::TailNodes::Nested(ident) => match &pos {
                    Position::Import(fname) => {
                        println!("ruda: importing {ident} from {fname}");
                        let root = identify_root(objects, &ident, Some(scopes), &fname, &node.1)?;
                        return traverse_tail(
                            objects,
                            tail,
                            context,
                            scopes,
                            code,
                            fun,
                            root,
                            scope_len,
                        );
                    }
                    Position::BinImport(fname) => {
                        println!("bin: importing {ident} from {fname}");
                        let root = identify_root(objects, &ident, Some(scopes), &fname, &node.1)?;
                        todo!("root: {root:?}");
                    }
                    Position::Variable(vname) => {
                        todo!()
                    }
                    Position::StructField(path, kind) => {
                        todo!()
                    }
                    Position::Function(_) => {
                        Err(CodegenError::CannotAttachMethodsToFunctions(node.1.clone()))?
                    }
                    Position::ReturnValue(val) => {
                        todo!()
                    }
                    Position::Pointer(ptr) => {
                        todo!()
                    }
                },
                expression_parser::TailNodes::Index(idx) => match &pos {
                    Position::BinImport(_) => Err(CodegenError::CannotIndexFile(node.1.clone()))?,
                    Position::Function(_) => {
                        Err(CodegenError::CannotIndexFunction(node.1.clone()))?
                    }
                    Position::StructField(_, _) => todo!(),
                    Position::Import(_) => Err(CodegenError::CannotIndexFile(node.1.clone()))?,
                    Position::Variable(var) => {
                        let var = match find_var(&scopes, &var) {
                            Some(var) => var,
                            None => {
                                Err(CodegenError::VariableNotFound(var.clone(), node.1.clone()))?
                            }
                        };
                        let pos_cloned = var.pos.clone();
                        let index_kind =
                            expression(objects, idx, scopes, code, context, &fun, scope_len)?;
                        code.push(Move(GENERAL_REG1, GENERAL_REG3));
                        code.read(&pos_cloned, GENERAL_REG1);
                    }
                    Position::Pointer(ptr) => {
                        todo!()
                    }
                    Position::ReturnValue(_) => todo!(),
                },
                expression_parser::TailNodes::Call(call_params) => {
                    match &pos {
                        Position::Function(fun_kind) => {
                            match fun_kind {
                                FunctionKind::Fun(fun_path) => {
                                    // legacy code
                                    // keep it in case there is a problem with function calls (copilot is cooking good :D)
                                    /*
                                    let mut temp_code = Code { code: Vec::new() };
                                    let mut called_fun = fun_path.get(objects)?;
                                    // setup stack TODO: avoid object creation
                                    let stack_len = match called_fun.stack_size {
                                        Some(len) => len,
                                        None => {
                                            gen_fun(objects, fun_path, context)?;
                                            called_fun = fun_path.get(objects)?;
                                            called_fun.stack_size.unwrap()
                                        }
                                    };
                                    *scope_len += 1;
                                    let obj = create_var_pos(scopes);
                                    temp_code
                                        .extend(&[Freeze, AllocateStatic(called_fun.args.len())]);
                                    temp_code.write(POINTER_REG, &obj);
                                    // setup args
                                    let mut args = Vec::new();
                                    for (idx, arg) in call_params.args.iter().enumerate() {
                                        let kind = expression(
                                            objects,
                                            arg,
                                            scopes,
                                            &mut temp_code,
                                            context,
                                            &fun,
                                            scope_len,
                                        )?;
                                        args.push(kind);
                                        temp_code.read(&obj, POINTER_REG);
                                        temp_code
                                            .extend(&[IndexStatic(idx), WritePtr(GENERAL_REG1)])
                                    }
                                    // type check
                                    let called_fun = fun_path.get(objects)?;
                                    for (idx, arg) in args.iter().enumerate() {
                                        let cmp = called_fun.args[idx].kind.cmp(&arg);
                                        if cmp.is_not_equal() {
                                            return Err(CodegenError::ArgTypeMismatch(
                                                called_fun.args[idx].clone(),
                                                arg.clone(),
                                                cmp,
                                                node.1.clone(),
                                            ));
                                        }
                                    }
                                    // move args
                                    temp_code.extend(&[ReserveStack(
                                        called_fun.stack_size.unwrap(),
                                        called_fun.pointers.unwrap_or(0),
                                    )]);
                                    if args.len() > 0 {
                                        temp_code.read(&obj, POINTER_REG);
                                    }
                                    for (idx, _) in args.iter().enumerate() {
                                        temp_code.extend(&[
                                            // moves ptr by 1
                                            IndexStatic(1),
                                            ReadPtr(GENERAL_REG1),
                                            Write(idx + 1, GENERAL_REG1),
                                        ]);
                                    }
                                    // call
                                    temp_code.extend(&[
                                        Jump(called_fun.location.unwrap()),
                                        // Unfreeze,
                                        Move(RETURN_REG, GENERAL_REG1),
                                    ]);
                                    merge_code(&mut code.code, &temp_code.code, stack_len);
                                    return Ok(Position::ReturnValue(
                                        called_fun.return_type.clone().unwrap_or(
                                            ShTypeBuilder::new().set_name("null").build(),
                                        ),
                                    ));
                                    */
                                    call_fun(
                                        objects,
                                        fun_path,
                                        context,
                                        scopes,
                                        code,
                                        scope_len,
                                        call_params,
                                        &node.1,
                                    )?;
                                }
                                FunctionKind::Binary(fun) => todo!("binary function call"),
                                FunctionKind::Dynamic(fun) => todo!("dynamic function call"),
                            };
                        }
                        _ => Err(CodegenError::CanCallOnlyFunctions(node.1.clone()))?,
                    }
                }
                expression_parser::TailNodes::Cast(_) => todo!(),
            }
        }
        None => {
            // finish the sequence
        }
    }
    Ok(pos)
}

fn call_binary(
    objects: &mut Context,
    fun: &InnerPath,
    context: &mut runtime_types::Context,
    scopes: &mut Vec<ScopeCached>,
    code: &mut Code,
    scope_len: &mut usize,
    call_params: &FunctionCall,
) -> Result<ShallowType, CodegenError> {
    use Instructions::*;
    let mut temp_code = Code { code: Vec::new() };
    let mut called_fun = fun.get_bin(objects)?;
    // setup arguments (stack is not needed)
    let args = called_fun.args.len();
    let mut args = Vec::new();
    for (idx, arg) in call_params.args.iter().enumerate() {
        let kind = expression(
            objects,
            arg,
            scopes,
            &mut temp_code,
            context,
            &fun,
            scope_len,
        )?;
        args.push(kind);
    }

    
    todo!("binary function call")
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
) -> Result<ShallowType, CodegenError> {
    use Instructions::*;
    let mut temp_code = Code { code: Vec::new() };
    let mut called_fun = fun.get(objects)?;
    // setup stack TODO: avoid object creation
    let stack_len = match called_fun.stack_size {
        Some(len) => len,
        None => {
            gen_fun(objects, fun, context)?;
            called_fun = fun.get(objects)?;
            called_fun.stack_size.unwrap()
        }
    };
    *scope_len += 1;
    let obj = create_var_pos(scopes);
    let scopes_len = scopes.len();
    scopes[scopes_len-1].variables.insert(
        scope_len.to_string(),
        Variable {
            kind: Some(ShallowType::empty()),
            pos: obj.clone(),
            value: None,
            line: line.clone(),
        },
    );
    temp_code.extend(&[Freeze, AllocateStatic(called_fun.args.len())]);
    temp_code.write(POINTER_REG, &obj);
    // setup args
    let mut args = Vec::new();
    for (idx, arg) in call_params.args.iter().enumerate() {
        let kind = expression(
            objects,
            arg,
            scopes,
            &mut temp_code,
            context,
            &fun,
            scope_len,
        )?;
        args.push(kind);
        temp_code.read(&obj, POINTER_REG);
        temp_code.extend(&[IndexStatic(idx), WritePtr(GENERAL_REG1)])
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
                called_fun.args[idx].line.clone(),
            ));
        }
    }
    // move args
    let next_stack_size = called_fun.stack_size.unwrap();
    temp_code.extend(&[ReserveStack(
        next_stack_size,
        called_fun.pointers.unwrap_or(0),
    )]);
    if args.len() > 0 {
        temp_code.read(&obj.add(-(next_stack_size as i64)), MEMORY_REG3);
    }
    /*for (idx, _) in args.iter().enumerate() {
        temp_code.extend(&[
            // moves ptr by 1
            ReadPtr(GENERAL_REG1),
            IndexStatic(1),
            // tu to čte špatně a dělá mi to naschvál
            Write(idx+next_stack_size-1, GENERAL_REG1),
        ]);
    }*/
    // call
    temp_code.extend(&[
        Jump(called_fun.location.unwrap()),
        Unfreeze,
        Move(RETURN_REG, GENERAL_REG1),
    ]);
    merge_code(&mut code.code, &temp_code.code, stack_len);
    Ok(called_fun
        .return_type
        .clone()
        .unwrap_or(ShTypeBuilder::new().set_name("null").build()))
}

fn get_scope(
    objects: &mut Context,
    block: &Vec<Nodes>,
    context: &mut runtime_types::Context,
    other_scopes: &mut Vec<ScopeCached>,
    code: &mut Code,
    fun: &InnerPath,
) -> Result<usize, CodegenError> {
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
            let scope_len = get_scope(objects, $block, context, other_scopes, $code, fun)?;
            other_scopes.pop();
            max_scope_len += scope_len;
            scope_len
        }};
    }

    let bool_type = ShTypeBuilder::new().set_name("bool").build();

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
                let expr_kind = match expr {
                    Some(expr) => {
                        let expr_kind = expression(
                            objects,
                            expr,
                            other_scopes,
                            code,
                            context,
                            &fun,
                            &mut max_scope_len,
                        )?;
                        code.write(GENERAL_REG1, &pos);
                        expr_kind
                    }
                    None => {
                        if let Some(kind) = kind {
                            if !kind.nullable {
                                Err(CodegenError::TypeNotNullable(line.clone()))?;
                            }
                        } else {
                            Err(CodegenError::NotInitializedNoType(line.clone()))?;
                        }
                        let null = new_const(context, &ConstValue::Null)?;
                        use Instructions::*;
                        code.extend(&[ReadConst(null, GENERAL_REG1)]);
                        code.write(GENERAL_REG1, &pos);
                        ShTypeBuilder::new().set_name("null").build()
                    }
                };
                let kind = match kind {
                    Some(kind) => kind.clone(),
                    None => expr_kind.clone(),
                };
                let cmp = kind.cmp(&expr_kind);
                if cmp.is_not_equal() {
                    return Err(CodegenError::VariableTypeMismatch(
                        kind.clone(),
                        expr_kind,
                        cmp,
                        line.clone(),
                    ));
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
                // if
                let mut expr_code = Code { code: Vec::new() };
                let mut block_code = Code { code: Vec::new() };
                let mut gotos = Vec::new();
                let kind = expression(
                    objects,
                    cond,
                    other_scopes,
                    &mut expr_code,
                    context,
                    &fun,
                    &mut max_scope_len,
                )?;
                if kind.cmp(&bool_type).is_not_equal() {
                    return Err(CodegenError::ExpectedBool(line.clone()));
                }
                let scope = open_scope!(body, &mut block_code);
                expr_code.push(Branch(
                    expr_code.code.len() + 1,
                    expr_code.code.len() + block_code.code.len() + 2,
                ));
                // elifs
                let mut elifs = Vec::new();
                for elif in elif {
                    let mut expr_code = Code { code: Vec::new() };
                    let mut block_code = Code { code: Vec::new() };
                    let kind = expression(
                        objects,
                        &elif.0,
                        other_scopes,
                        &mut expr_code,
                        context,
                        &fun,
                        &mut max_scope_len,
                    )?;
                    if kind.cmp(&bool_type).is_not_equal() {
                        return Err(CodegenError::ExpectedBool(elif.2.clone()));
                    }
                    let scope = open_scope!(&elif.1, &mut block_code);
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
                        let scope = open_scope!(&els.0, &mut block_code);
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
            }
            crate::codeblock_parser::Nodes::While { cond, body, line } => {
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
                )?;
                if kind.cmp(&bool_type).is_not_equal() {
                    return Err(CodegenError::ExpectedBool(line.clone()));
                }
                let scope = open_scope!(body, &mut block_code);
                expr_code.push(Branch(
                    expr_code.code.len() + 1,
                    expr_code.code.len() + block_code.code.len() + 2,
                ));
                let mut buffer = Vec::new();
                merge_code(&mut buffer, &expr_code.code, scope);
                merge_code(&mut buffer, &block_code.code, scope);
                buffer.push(Goto(0));
                merge_code(&mut code.code, &buffer, scope);
            }
            crate::codeblock_parser::Nodes::For {
                ident,
                expr,
                body,
                line,
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
                        )?;
                        expr_code.push(Move(GENERAL_REG1, RETURN_REG));
                        kind
                    }
                    None => {
                        expr_code.push(ReadConst(0, RETURN_REG));
                        ShTypeBuilder::new().set_name("null").build()
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
                        ShTypeBuilder::new().set_name("null").build(),
                        kind,
                        TypeComparison::NotEqual,
                        line.clone(),
                    ));
                }
                expr_code.push(Instructions::Return);
                merge_code(&mut code.code, &expr_code.code, 0);
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
                )?;
            }
            crate::codeblock_parser::Nodes::Block { body, line } => {
                let scope = open_scope!(body, code);
            }
            crate::codeblock_parser::Nodes::Break { line } => todo!(),
            crate::codeblock_parser::Nodes::Continue { line } => todo!(),
            crate::codeblock_parser::Nodes::Loop { body, line } => {
                use Instructions::*;
                let mut block_code = Code { code: Vec::new() };
                let scope = open_scope!(body, &mut block_code);
                block_code.push(Goto(0));
                merge_code(&mut context.code.data, &block_code.code, scope);
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
            } => todo!(),
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
                        let root = identify_root(
                            objects,
                            &val.root.0,
                            Some(other_scopes),
                            &fun.file,
                            &val.root.1,
                        )?;
                        let pos = traverse_tail(
                            objects,
                            &mut val.tail.iter(),
                            context,
                            other_scopes,
                            &mut target_code,
                            fun,
                            root,
                            &mut max_scope_len,
                        )?;
                        match pos {
                            Position::Variable(var) => {
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
                                )?;
                                if let Operators::Equal = op {
                                    if let Some(kind) = &var.kind {
                                        let cmp = kind.cmp(&expr);
                                        if cmp.is_not_equal() {
                                            return Err(CodegenError::VariableTypeMismatch(
                                                kind.clone(),
                                                expr,
                                                cmp,
                                                line.clone(),
                                            ));
                                        }
                                    }
                                    conclusion_code.write(GENERAL_REG1, &var.pos);
                                } else {
                                    todo!("other operators (need basic operators first)")
                                }
                                merge_code(&mut code.code, &target_code.code, 0);
                                merge_code(&mut code.code, &expr_code.code, 0);
                                merge_code(&mut code.code, &conclusion_code.code, 0);
                            }
                            Position::Pointer(_) => todo!(),
                            _ => todo!(),
                        }
                    }
                    ValueType::Parenthesis(_, _) => todo!(),
                    _ => {
                        unreachable!(
                            "target not handled properly by the compiler, please report this bug"
                        )
                    }
                }
            }
        }
    }
    Ok(max_scope_len)
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
    scope_len: usize,
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

impl ScopeCached {
    pub fn insert_dummy(&mut self, pos: MemoryTypes) {
        self.variables.insert(
            self.variables.len().to_string(),
            Variable {
                kind: None,
                pos,
                value: None,
                line: Line { line: 0, column: 0 },
            },
        );
    }
}

#[derive(Debug, Clone)]
struct Variable {
    kind: Option<ShallowType>,
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
    pub fn extend_start(&mut self, other: &[Instructions]) {
        let mut code = Vec::new();
        code.extend(other);
        code.extend(&self.code);
        self.code = code;
    }
    pub fn push_start(&mut self, instr: Instructions) {
        let mut code = vec![instr];
        code.extend(&self.code);
        self.code = code;
    }
}

#[derive(Debug, Clone)]
enum StructField {
    Field(String),
    Method(String),
    OverloadedOperator(Operators),
    TraitMethod(String, String),
}

#[derive(Debug, Clone)]
enum Position {
    Function(FunctionKind),
    StructField(InnerPath, StructField),
    Import(String),
    BinImport(String),
    Variable(String),
    Pointer(ShallowType),
    ReturnValue(ShallowType),
}

#[derive(Debug, Clone)]
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
            Some(b) => Err(CodegenError::FunctionNotFound(self.clone())),
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
            Some(b) => Err(CodegenError::FunctionNotFound(self.clone())),
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
        objects: &'a mut Context,
    ) -> Result<&libloader::Function, CodegenError> {
        let file = match objects.1.get(&self.file) {
            Some(f) => f,
            None => {
                return Err(CodegenError::FunctionNotFound(self.clone()));
            }
        };
        match &self.block {
            Some(b) => Err(CodegenError::FunctionNotFound(self.clone())),
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
