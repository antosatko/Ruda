use runtime::runtime_types::{CODE_PTR_REG, self};

use crate::{prep_objects::Context, intermediate};

pub fn gen(objects: &Context) -> Result<runtime::runtime_types::Context, CodegenError> {
    let mut vm_context = runtime::runtime_types::Context::new();

    let code = &mut vm_context.code.data;
    // hello world (for testing)
    /*{
        use runtime::runtime_types::Instructions;
        vm_context
            .memory
            .stack
            .data
            .push(runtime::runtime_types::Types::Pointer(
                vm_context.memory.strings.from_str("Hello, world!"), // position of string in memory
                runtime::runtime_types::PointerTypes::String, // type of string
            ));
        let instrs = vec![
            Instructions::ReadConst(0, runtime::runtime_types::POINTER_REG), // read string from stack
            Instructions::Cal(0, 1), // call print
        ];
        code.extend(instrs);
    }*/
    code.push(runtime::runtime_types::Instructions::End);
    Ok(vm_context)
}

pub enum CodegenError {}

pub fn stringify(
    context: &runtime::runtime_types::Context,
    shlibs: &Vec<stringify::ShLib>,
) -> String {
    use stringify::stringify;
    stringify(context, Some(&shlibs))
}

fn gen_fun(objects: &Context, fun: &intermediate::dictionary::Function, context: &mut runtime_types::Context) {
    let mut code: Vec<runtime::runtime_types::Instructions> = Vec::new();

}

/// returns starting point and length of appended code
fn merge_code(context: &mut runtime_types::Context, new_code: &Vec<runtime::runtime_types::Instructions>) -> (usize, usize) {
    let code = &mut context.code.data;
    code.reserve(new_code.len());
    let instrs = code.len();
    let consts = context.memory.stack.data.len();
    for instr in new_code {
        use runtime::runtime_types::Instructions::*;
        let instr = match instr {
            ReadConst(idx, reg) => {
                let idx = *idx + consts;
                let reg = *reg;
                ReadConst(idx, reg)
            }
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
            _ => *instr
        };
        code.push(instr);
    };
    (instrs, code.len() - instrs)
}