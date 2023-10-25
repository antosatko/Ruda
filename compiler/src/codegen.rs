use std::collections::HashMap;
use std::thread::Scope;

use runtime::runtime_types::{
    self, Instructions, Stack, CODE_PTR_REG, GENERAL_REG1, POINTER_REG, RETURN_REG,
};

use crate::codeblock_parser::Nodes;
use crate::expression_parser::{self, ValueType};
use crate::intermediate::dictionary::{ConstValue, Function, ShallowType};
use crate::tree_walker::tree_walker::Line;
use crate::{intermediate, prep_objects::Context};

use crate::libloader::{MemoryTypes, Registers};

pub fn gen(objects: &Context, main: &str) -> Result<runtime::runtime_types::Context, CodegenError> {
    let mut vm_context = runtime::runtime_types::Context::new();
    gen_fun(objects, objects.get_main(), &mut vm_context);
    vm_context.code.data.push(Instructions::End);
    Ok(vm_context)
}

pub enum CodegenError {
    CannotInitializeConstant,
}

pub fn stringify(
    context: &runtime::runtime_types::Context,
    shlibs: &Vec<stringify::ShLib>,
) -> String {
    use stringify::stringify;
    stringify(context, Some(&shlibs))
}

fn gen_fun(
    objects: &Context,
    fun: &intermediate::dictionary::Function,
    context: &mut runtime_types::Context,
) -> (usize, usize) {
    let mut code = Code { code: Vec::new() };
    let mut scopes = Vec::new();
    get_scope(objects, &fun.code, context, &mut scopes, &mut code);

    merge_code(context, &code.code)
}

/// evaluates expression at runtime and puts result in reg1
fn expression(
    objects: &Context,
    expr: &expression_parser::ValueType,
    other_scopes: &mut Vec<Cached>,
    code: &mut Code,
    context: &mut runtime_types::Context,
) -> Result<(), CodegenError> {
    use Instructions::*;
    match expr {
        ValueType::Literal(lit) => match &lit.value {
            expression_parser::Literals::Number(_) => todo!(),
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
                                arr_.push(expression(objects, value, other_scopes, code, context)?);
                            }
                        }
                    }
                };
            }
            expression_parser::Literals::String(str) => {
                let pos = new_const(context, &ConstValue::String(str.clone()))?;
                code.extend(&[
                    ReadConst(pos, POINTER_REG),
                    Cal(1, 3),
                    Move(RETURN_REG, GENERAL_REG1),
                ]);
            }
            expression_parser::Literals::Char(c) => {
                let pos = new_const(context, &ConstValue::Char(*c))?;
                code.push(ReadConst(pos, GENERAL_REG1));
            }
        },
        ValueType::AnonymousFunction(_) => todo!(),
        ValueType::Parenthesis(_, _) => todo!(),
        ValueType::Expression(_) => todo!(),
        ValueType::Operator(_, _) => {unreachable!("operator not handled properly by the compiler, please report this bug")},
        ValueType::Value(_) => todo!(),
        ValueType::Blank => {}
    }
    Ok(())
}

fn get_scope(
    objects: &Context,
    block: &Vec<Nodes>,
    context: &mut runtime_types::Context,
    other_scopes: &mut Vec<Cached>,
    code: &mut Code,
) {
    other_scopes.push(Cached {
        variables: HashMap::new(),
    });
    macro_rules! last {
        ($arr: expr) => {{
            let len = $arr.len();
            &mut $arr[len - 1]
        }};
    }

    for node in block {
        match node {
            crate::codeblock_parser::Nodes::Let {
                ident,
                expr,
                kind,
                line,
            } => {
                let pos = create_var_pos(&other_scopes);
                let cach = last!(other_scopes);
                cach.variables.insert(
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
            } => todo!(),
            crate::codeblock_parser::Nodes::While { cond, body, line } => todo!(),
            crate::codeblock_parser::Nodes::For {
                ident,
                expr,
                body,
                line,
            } => todo!(),
            crate::codeblock_parser::Nodes::Return { expr, line } => todo!(),
            crate::codeblock_parser::Nodes::Expr { expr, line } => {
                expression(objects, expr, other_scopes, code, context);
            },
            crate::codeblock_parser::Nodes::Block { body, line } => todo!(),
            crate::codeblock_parser::Nodes::Break { line } => todo!(),
            crate::codeblock_parser::Nodes::Continue { line } => todo!(),
            crate::codeblock_parser::Nodes::Loop { body, line } => todo!(),
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
            } => todo!(),
        }
    }
}

fn create_var_pos(scopes: &Vec<Cached>) -> MemoryTypes {
    let len = {
        let mut len = 0;
        for scope in scopes {
            len += scope.variables.len();
        }
        len
    };
    match len {
        0 => MemoryTypes::Register(Registers::G4),
        1 => MemoryTypes::Register(Registers::G5),
        2 => MemoryTypes::Register(Registers::G6),
        _ => MemoryTypes::Stack(len - 3),
    }
}

/// TODO: todo xd
fn try_get_const_val() -> Option<ConstValue> {
    None
}

/// returns starting point and length of appended code
fn merge_code(
    context: &mut runtime_types::Context,
    new_code: &Vec<Instructions>,
) -> (usize, usize) {
    let code = &mut context.code.data;
    code.reserve(new_code.len());
    let instrs = code.len();
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
        code.push(instr);
    }
    (instrs, code.len() - instrs)
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
        _ => {
            context.memory.stack.data.push(value.to_runtime());
        }
        intermediate::dictionary::ConstValue::Undefined => {
            Err(CodegenError::CannotInitializeConstant)?
        }
    }
    Ok(idx)
}

struct Cached {
    variables: HashMap<String, Variable>,
}

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
}
