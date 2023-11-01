use std::collections::HashMap;
use std::net;
use std::thread::Scope;

use runtime::runtime_types::{
    self, Instructions, Stack, Types, CODE_PTR_REG, GENERAL_REG1, POINTER_REG, RETURN_REG,
};

use crate::codeblock_parser::Nodes;
use crate::expression_parser::{self, ValueType};
use crate::intermediate::dictionary::{
    ConstValue, Function, ShTypeBuilder, ShallowType, TypeComparison, self,
};
use crate::lexer::tokenizer::Tokens;
use crate::tree_walker::tree_walker::Line;
use crate::{intermediate, prep_objects::Context};

use crate::libloader::{MemoryTypes, Registers};

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
    let main = gen_fun(objects, &main_path, &mut vm_context)?;
    call_main(&main.clone(), &mut vm_context)?;
    Ok(vm_context)
}

fn call_main(main: &Function, context: &mut runtime_types::Context) -> Result<(), CodegenError> {
    use Instructions::*;
    context.code.entry_point = context.code.data.len() + 1;
    let consts_len = context.memory.stack.data.len();
    context.code.data.extend(&[
        End,
        ReserveStack(
            main.stack_size.unwrap() + consts_len,
            main.pointers.unwrap_or(0),
        ),
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
) -> Result<&'a mut Function, CodegenError> {
    let context_code_end = context.code.data.len();
    let mut code = Code { code: Vec::new() };
    let mut scopes = Vec::new();
    let this_fun = fun.get(objects)?;
    let scope_len = get_scope(
        objects,
        &this_fun.code,
        context,
        &mut scopes,
        &mut code,
        fun,
    )?;
    flip_stack_access(scope_len, &mut code);
    let pos = merge_code(&mut context.code.data, &code.code, scope_len);
    let this_fun = fun.get_mut(objects)?;
    this_fun.location = Some(context_code_end);
    this_fun.stack_size = Some(scope_len);
    Ok(this_fun)
}

/// evaluates expression at runtime and puts result in reg1
fn expression(
    objects: &Context,
    expr: &expression_parser::ValueType,
    scopes: &mut Vec<ScopeCached>,
    code: &mut Code,
    context: &mut runtime_types::Context,
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
                                    arr_.push(expression(objects, value, scopes, code, context)?);
                                }
                            }
                        }
                    };
                }
                expression_parser::Literals::String(str) => {
                    let pos = new_const(context, &ConstValue::String(str.clone()))?;
                    println!("refs: {:?}", lit.refs);
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
            let kind = expression(objects, expr, scopes, code, context)?;
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
            match find_var(scopes, &value.root.0) {
                Some(var) => {
                    let iter = value.tail.iter();
                    let pos = var.pos;
                    for (node, line) in iter {
                        match node {
                            expression_parser::TailNodes::Nested(_) => todo!(),
                            expression_parser::TailNodes::Index(_) => todo!(),
                            expression_parser::TailNodes::Call(_) => todo!(),
                            expression_parser::TailNodes::Cast(_) => todo!(),
                        }
                    }
                    match value.refs {
                        expression_parser::Ref::Dereferencing(_) => todo!(),
                        expression_parser::Ref::Reference(_) => todo!(),
                        expression_parser::Ref::None => {
                            code.read(&pos, GENERAL_REG1);
                        }
                    }
                }
                None => Err(CodegenError::VariableNotFound(
                    value.root.0.clone(),
                    value.root.1.clone(),
                ))?,
            }
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

fn find_import<'a>(objects: &'a Context, ident: &'a str, file_name: &'a str) -> Option<&'a str> {
    let file = match objects.0.get(file_name) {
        Some(dictionary) => {
            for import in dictionary.imports.iter() {
                if import.alias == ident {
                    return Some(&import.path);
                }
            }
        },
        None => None?,
    };
    None
}

fn gen_value(
    objects: &Context,
    value: &expression_parser::Variable,
    context: &mut runtime_types::Context,
    scopes: &mut Vec<ScopeCached>,
    code: &mut Code,
    fun: &InnerPath,
) -> Result<ShallowType, CodegenError> {
    use Instructions::*;
    let mut return_kind = ShallowType::empty();
    if let Some(var) = find_var(scopes, &value.root.0) {
        let iter: std::slice::Iter<'_, (expression_parser::TailNodes, Line)> = value.tail.iter();
        let root = identify_root(objects, iter, context, scopes, code, fun)?;
        let (kind, pos) = traverse_tail(objects, iter, context, scopes, code, fun, root)?;
        
    }
    Ok(return_kind)
}

fn identify_root(
    objects: &Context,
    tail: std::slice::Iter<'_, (expression_parser::TailNodes, Line)>,
    context: &mut runtime_types::Context,
    scopes: &mut Vec<ScopeCached>,
    code: &mut Code,
    fun: &InnerPath,
) -> Result<(ShallowType, Position), CodegenError> {

}

fn traverse_tail(
    objects: &Context,
    tail: std::slice::Iter<'_, (expression_parser::TailNodes, Line)>,
    context: &mut runtime_types::Context,
    scopes: &mut Vec<ScopeCached>,
    code: &mut Code,
    fun: &InnerPath,
    (kind, pos): (ShallowType, Position),
) -> Result<(ShallowType, Position), CodegenError> {
    use Instructions::*;
    let mut return_kind = ShallowType::empty();
    for (node, line) in tail {
        match node {
            expression_parser::TailNodes::Nested(_) => todo!(),
            expression_parser::TailNodes::Index(_) => todo!(),
            expression_parser::TailNodes::Call(_) => todo!(),
            expression_parser::TailNodes::Cast(_) => todo!(),
        }
    }
    Ok((kind, pos))
}

fn get_scope(
    objects: &Context,
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
                let pos: MemoryTypes = create_var_pos(&other_scopes);
                let expr_kind = match expr {
                    Some(expr) => {
                        let expr_kind = expression(objects, expr, other_scopes, code, context)?;
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
                if let Some(kind) = kind {
                    let cmp = kind.cmp(&expr_kind);
                    if cmp.is_not_equal() {
                        return Err(CodegenError::VariableTypeMismatch(
                            kind.clone(),
                            expr_kind,
                            cmp,
                            line.clone(),
                        ));
                    }
                }
                let cache = last!(other_scopes);
                cache.variables.insert(
                    ident.clone(),
                    Variable {
                        kind: kind.clone(),
                        pos,
                        value: try_get_const_val(),
                        line: line.clone(),
                    },
                );
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
                let kind = expression(objects, cond, other_scopes, &mut expr_code, context)?;
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
                    let kind = expression(objects, &elif.0, other_scopes, &mut expr_code, context)?;
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
                let kind = expression(objects, cond, other_scopes, &mut expr_code, context)?;
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
                let kind = match expr {
                    Some(expr) => {
                        let kind = expression(objects, expr, other_scopes, code, context)?;
                        code.push(Move(GENERAL_REG1, RETURN_REG));
                        kind
                    }
                    None => {
                        code.push(ReadConst(0, RETURN_REG));
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
                code.extend(&[Return]);
            }
            crate::codeblock_parser::Nodes::Expr { expr, line } => {
                expression(objects, expr, other_scopes, code, context)?;
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
                // let expr_kind = expression(objects, expr, other_scopes, &mut expr_code, context)?;
            }
        }
    }
    Ok(max_scope_len)
}

fn call_fun(
    objects: &Context,
    fun: &InnerPath,
    args: &Vec<expression_parser::ValueType>,
    scopes: &mut Vec<ScopeCached>,
    code: &mut Code,
    context: &mut runtime_types::Context,
) -> Result<(), CodegenError> {
    let fun = fun.get(objects)?;
    use Instructions::*;
    code.extend(&[
        ReserveStack(fun.stack_size.unwrap(), fun.pointers.unwrap_or(0)),
        Jump(fun.location.unwrap()),
    ]);
    Ok(())
}
fn create_var_pos(scopes: &Vec<ScopeCached>) -> MemoryTypes {
    let len = {
        let mut len = 0;
        for scope in scopes {
            len += scope.variables.len();
        }
        len
    };
    MemoryTypes::Stack(len + 1)
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
            Jump(pos) => {
                let pos = *pos + instrs;
                Jump(pos)
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
                *from = len - *from + 1;
            }
            Write(to, _) => {
                *to = len - *to + 1;
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

struct ScopeCached {
    variables: HashMap<String, Variable>,
}

#[derive(Debug)]
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

enum StructField {
    Field(usize),
    Method(usize),
    OverloadedOperator(usize),
    TraitMethod(usize),
}

enum Position {
    Function(InnerPath),
    StructField(InnerPath, StructField),
    File(String),
    Variable(String),
}

#[derive(Debug, Clone)]
pub struct InnerPath {
    file: String,
    block: Option<String>,
    ident: String,
}

impl InnerPath {
    pub fn main() -> Self {
        InnerPath {
            file: "main.rd".to_string(),
            block: None,
            ident: "main".to_string(),
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
}
