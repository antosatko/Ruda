use runtime::runtime_types::{Types, REGISTER_SIZE, Context, Instructions};

use crate::codegen::{Code, CodegenError};
use crate::libloader::MemoryTypes;

pub fn optimize(code: &mut Code, ctx: &Context) -> Result<Code, CodegenError> {
    let mut state = State::new(code, ctx);
    state.process();
    Ok(state.new_code)
}


/// The state of the optimizer.
/// This will hold information about locations of different values in memory and such.
struct State<'a> {
    cursor: usize,
    values: Vec<Value>,
    /// Values that are known to be in registers
    registers: [Option<Types>; REGISTER_SIZE],
    /// Values that are known to be on the stack
    stack: Vec<Option<Types>>,
    code: &'a Code,
    new_code: Code,
    ctx: &'a Context,
}

impl State<'_> {
    fn new<'a>(code: &'a Code, ctx: &'a Context) -> State<'a> {
        State {
            cursor: 0,
            values: Vec::new(),
            code,
            new_code: Code::new(),
            registers: [None; REGISTER_SIZE],
            stack: Vec::new(),
            ctx,
        }
    }

    pub fn process(&mut self) {
        while self.cursor < self.code.code.len() {
            self.process_instruction();
        }
    }

    fn process_instruction(&mut self) {
        let instruction = self.code.code[self.cursor];
        match instruction {
            _ => {
                self.default();
            }
        }
    }

    fn default(&mut self) {
        self.new_code.code.push(self.code.code[self.cursor]);
        self.cursor += 1;
    }
}

struct Value {
    /// all the locations where this value is currently stored
    locations: Vec<MemoryTypes>,
}