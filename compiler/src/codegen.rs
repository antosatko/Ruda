use runtime::runtime_types::CODE_PTR_REG;

use crate::prep_objects::Context;



pub fn gen(context: &mut Context) -> Result<runtime::runtime_types::Context, CodegenError> {
    let mut vm_context = runtime::runtime_types::Context::new();
    
    
    
    let code = &mut vm_context.code.data;
    // hello world (for testing)
    {
        use runtime::runtime_types::Instructions;
        let instrs = vec![
            Instructions::ReadConst(0, 6),
            Instructions::Cal(0, 1),
        ];
        code.extend(instrs);
        vm_context.memory.strings.new();
        vm_context.memory.strings.pool[0] = "Hello, world!".to_string();
        vm_context.memory.stack.data.push(runtime::runtime_types::Types::Pointer(0, runtime::runtime_types::PointerTypes::String));
    }
    code.push(runtime::runtime_types::Instructions::End);
    Ok(vm_context)
}

pub enum CodegenError {
    
}

pub fn stringify(context: &runtime::runtime_types::Context, shlibs: &Vec<stringify::ShLib>) -> String {
    use stringify::stringify;
    stringify(context, Some(&shlibs))
}