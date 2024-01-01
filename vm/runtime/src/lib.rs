use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;
use std::vec;

use runtime_error::*;
use runtime_types::*;

macro_rules! panic_msg {
    () => {
        std::panic::set_hook(Box::new(|info| {
            println!("{}", info.to_string());

            println!("This is most likely caused by a bug in the compiler, please report it to the compiler developer.");

            println!("If you are the compiler developer, then you know what to do :)");
        }));
    };
}

impl Context {
    pub fn new(libs: Libs) -> Self {
        Self {
            memory: Memory {
                stack: Stack {
                    data: vec![],
                    ptr: 0,
                    call_stack: [CallStack {
                        end: 0,
                        code_ptr: 0,
                        reg_freeze: [Types::Null; FREEZED_REG_SIZE],
                        pointers_len: 0,
                    }; CALL_STACK_SIZE],
                },
                args: Args { data: [[Types::Null; MAX_ARGS]; CALL_STACK_SIZE], ptr: 1 },
                registers: [Types::Null; REGISTER_SIZE],
                heap: Heap {
                    data: vec![],
                    garbage: vec![],
                },
                strings: Strings {
                    pool: vec![],
                    garbage: vec![],
                    static_strings: 0,
                },
                non_primitives: vec![],
                gc: GarbageCollector {
                    sweeps_count: 0,
                    memory_swept: 0,
                    disabled: false,
                    sweep_threshold: usize::MAX,
                },
                fun_table: vec![],
                runtime_args: vec![],
                user_data: UserDataContainer {
                    data: vec![],
                    garbage: vec![],
                },
            },
            code: Code {
                data: vec![],
                ptr: 0,
                entry_point: 0,
            },

            break_code: None,
            catches: Catches {
                catches_ptr: 0,
                cache: [Catch {
                    code_ptr: 0,
                    id: None,
                    cs_ptr: 0,
                }; CALL_STACK_SIZE],
            },
            exit_code: ExitCodes::End,

            libs,
        }
    }
    /// runs the context
    pub fn run(&mut self) {
        panic_msg!();
        while self.read_line() {
            // TODO: remove for or testing
            // self.memory.gc_sweep_unoptimized()
        }
    }
    /// runs the context for a given number of cycles
    pub fn run_for(&mut self, cycles: usize) {
        panic_msg!();
        for _ in 0..cycles {
            if !self.read_line() {
                break;
            }
        }
    }
    /// runs the context as fast as possible without checking for errors
    ///
    /// This may cause undefined behaviour or crash the program unexpectedly
    /// Run at your own risk
    pub fn run_unchecked(&mut self) {
        panic_msg!();
        loop {
            self.read_line();
            self.read_line();
            self.read_line();
            self.read_line();
            self.read_line();
            self.read_line();
            self.read_line();
            self.read_line();
            self.read_line();
            if !self.read_line() {
                break;
            }
        }
    }
    /// runs the context while printing the actions
    pub fn run_debug(&mut self) {
        println!("Running in debug mode...");
        println!("{}", self.instruction_debug());
        while self.read_line() {
            println!("{}", self.instruction_debug());
        }
    }
    pub fn read_line(&mut self) -> bool {
        macro_rules! operation {
            (ptr, $operand: ident, $num1: ident, bool, $r1: expr, $r2: expr, $res: expr) => {
                if let Types::Pointer(num2, _) = self.memory.registers[$r2] {
                    self.memory.registers[$res] = Types::Bool($num1.$operand(&num2));
                } else {
                    return self.panic_rt(ErrTypes::CrossTypeOperation(
                        self.memory.registers[$r1],
                        self.memory.registers[$r2],
                        self.code.data[self.code.ptr],
                    ));
                }
            };
            ($type: tt, $operand: ident, $num1: ident, bool, $r1: expr, $r2: expr, $res: expr) => {
                if let Types::$type(num2) = self.memory.registers[$r2] {
                    self.memory.registers[$res] = Types::Bool($num1.$operand(&num2));
                } else {
                    return self.panic_rt(ErrTypes::CrossTypeOperation(
                        self.memory.registers[$r1],
                        self.memory.registers[$r2],
                        self.code.data[self.code.ptr],
                    ));
                }
            };
            ($type: tt, $operand: ident, $num1: ident, $r1: expr, $r2: expr, $res: expr) => {
                if let Types::$type(num2) = self.memory.registers[$r2] {
                    self.memory.registers[$res] = Types::$type($num1.$operand(num2));
                } else {
                    return self.panic_rt(ErrTypes::CrossTypeOperation(
                        self.memory.registers[$r1],
                        self.memory.registers[$r2],
                        self.code.data[self.code.ptr],
                    ));
                }
            };
            ($type: tt, %, $num1: ident, $r1: expr, $r2: expr, $res: expr) => {
                if let Types::$type(num2) = self.memory.registers[$r2] {
                    self.memory.registers[$res] = Types::$type($num1 % num2);
                } else {
                    return self.panic_rt(ErrTypes::CrossTypeOperation(
                        self.memory.registers[$r1],
                        self.memory.registers[$r2],
                        self.code.data[self.code.ptr],
                    ));
                }
            };
        }
        use Instructions::*;
        match self.code.data[self.code.ptr] {
            Write(stack_offset, register) => {
                let end = self.stack_end();
                self.memory.stack.data[end - stack_offset] = self.memory.registers[register];
                self.next_line();
            }
            Read(stack_offset, reg) => {
                // println!("{:?}", self.memory.stack.data);
                let end = self.stack_end();
                self.memory.registers[reg] = self.memory.stack.data[end - stack_offset];
                self.next_line();
            }
            WritePtr(value_reg) => {
                if let Types::Pointer(u_size, kind) = self.memory.registers[POINTER_REG] {
                    match kind {
                        PointerTypes::Stack => {
                            self.memory.stack.data[u_size] = self.memory.registers[value_reg];
                        }
                        PointerTypes::Heap(loc) => {
                            self.memory.heap.data[u_size][loc] = self.memory.registers[value_reg];
                        }
                        PointerTypes::Object => {
                            self.memory.heap.data[u_size][0] = self.memory.registers[value_reg];
                        }
                        PointerTypes::String => {
                            if let Types::Pointer(dest, PointerTypes::String) =
                                self.memory.registers[value_reg]
                            {
                                self.memory.strings.copy_from(u_size, dest)
                            } else {
                                return self.panic_rt(ErrTypes::Expected(
                                    Types::Pointer(0, PointerTypes::String),
                                    self.memory.registers[value_reg],
                                ));
                            }
                        }
                        PointerTypes::Char(loc) => {
                            if let Types::Char(chr) = self.memory.registers[value_reg] {
                                self.memory.strings.pool[u_size]
                                    .replace_range(loc..loc + 1, &chr.to_string());
                            } else {
                                return self.panic_rt(ErrTypes::Expected(
                                    Types::Char('a'),
                                    self.memory.registers[value_reg],
                                ));
                            }
                        }
                        PointerTypes::UserData => {
                            return self.panic_rt(ErrTypes::CannotReadUserdata);
                        }
                    }
                } else {
                    return self.panic_rt(ErrTypes::Expected(
                        Types::Pointer(0, PointerTypes::Heap(0)),
                        self.memory.registers[POINTER_REG],
                    ));
                }
                self.next_line();
            }
            ReadPtr(cash_reg) => {
                if let Types::Pointer(u_size, kind) = self.memory.registers[POINTER_REG] {
                    match kind {
                        PointerTypes::Stack => {
                            self.memory.registers[cash_reg] = self.memory.stack.data[u_size];
                        }
                        PointerTypes::Heap(idx) => {
                            self.memory.registers[cash_reg] = self.memory.heap.data[u_size][idx];
                        }
                        PointerTypes::Object => {
                            self.memory.registers[cash_reg] = self.memory.heap.data[u_size][0];
                        }
                        PointerTypes::String => {
                            return self.panic_rt(ErrTypes::InvalidType(
                                self.memory.registers[POINTER_REG],
                                Types::Pointer(0, PointerTypes::Heap(0)),
                            ));
                        }
                        PointerTypes::Char(idx) => {
                            self.memory.registers[cash_reg] = Types::Char(
                                self.memory.strings.pool[u_size].chars().nth(idx).unwrap(),
                            );
                        }
                        PointerTypes::UserData => {
                            return self.panic_rt(ErrTypes::CannotReadUserdata);
                        }
                    }
                } else {
                    return self.panic_rt(ErrTypes::InvalidType(
                        self.memory.registers[POINTER_REG],
                        Types::Pointer(0, PointerTypes::Heap(0)),
                    ));
                }
                self.next_line();
            }
            ReadConst(stack_pos, reg) => {
                self.memory.registers[reg] = self.memory.stack.data[stack_pos];
                self.next_line();
            }
            Ptr(stack_offset) => {
                self.memory.registers[GENERAL_REG1] =
                    Types::Pointer(self.stack_end() - stack_offset, PointerTypes::Stack);
                self.next_line();
            }
            Index(index_reg) => {
                if let Types::Pointer(u_size, kind) = self.memory.registers[POINTER_REG] {
                    if let Types::Uint(index) = self.memory.registers[index_reg] {
                        match kind {
                            PointerTypes::Object => {
                                self.memory.registers[POINTER_REG] =
                                    Types::Pointer(u_size, PointerTypes::Heap(index));
                            }
                            PointerTypes::Stack => {
                                self.memory.registers[POINTER_REG] =
                                    Types::Pointer(u_size + index, PointerTypes::Stack);
                            }
                            PointerTypes::Heap(_) => {
                                return self.panic_rt(ErrTypes::WrongTypeOperation(
                                    self.memory.registers[POINTER_REG],
                                    self.code.data[self.code.ptr],
                                ));
                            }
                            PointerTypes::Char(_) => {
                                return self.panic_rt(ErrTypes::WrongTypeOperation(
                                    self.memory.registers[POINTER_REG],
                                    self.code.data[self.code.ptr],
                                ));
                            }
                            PointerTypes::String => {
                                self.memory.registers[POINTER_REG] =
                                    Types::Pointer(u_size, PointerTypes::Char(index));
                            }
                            PointerTypes::UserData => {
                                return self.panic_rt(ErrTypes::CannotReadUserdata);
                            }
                        }
                    } else {
                        return self.panic_rt(ErrTypes::WrongTypeOperation(
                            self.memory.registers[POINTER_REG],
                            self.code.data[self.code.ptr],
                        ));
                    }
                } else {
                    return self.panic_rt(ErrTypes::WrongTypeOperation(
                        self.memory.registers[POINTER_REG],
                        self.code.data[self.code.ptr],
                    ));
                }
                self.next_line();
            }
            IndexStatic(index) => {
                if let Types::Pointer(u_size, kind) = self.memory.registers[POINTER_REG] {
                    match kind {
                        PointerTypes::Object => {
                            self.memory.registers[POINTER_REG] =
                                Types::Pointer(u_size, PointerTypes::Heap(index));
                        }
                        PointerTypes::Stack => {
                            self.memory.registers[POINTER_REG] =
                                Types::Pointer(u_size + index, PointerTypes::Stack);
                        }
                        PointerTypes::Heap(loc) => {
                            self.memory.registers[POINTER_REG] =
                                Types::Pointer(u_size, PointerTypes::Heap(loc + index));
                        }
                        PointerTypes::Char(_) => {
                            return self.panic_rt(ErrTypes::WrongTypeOperation(
                                self.memory.registers[POINTER_REG],
                                self.code.data[self.code.ptr],
                            ));
                        }
                        PointerTypes::String => {
                            self.memory.registers[POINTER_REG] =
                                Types::Pointer(u_size, PointerTypes::Char(index));
                        }
                        PointerTypes::UserData => {
                            return self.panic_rt(ErrTypes::CannotReadUserdata);
                        }
                    }
                } else {
                    return self.panic_rt(ErrTypes::WrongTypeOperation(
                        self.memory.registers[POINTER_REG],
                        self.code.data[self.code.ptr],
                    ));
                }
                self.next_line();
            }
            Allocate(size_reg) => {
                if let Types::Uint(size) = self.memory.registers[size_reg] {
                    self.memory.registers[POINTER_REG] =
                        Types::Pointer(self.memory.allocate_obj(size), PointerTypes::Object);
                } else {
                    return self.panic_rt(ErrTypes::Expected(
                        Types::Uint(0),
                        self.memory.registers[size_reg],
                    ));
                }
                self.next_line();
            }
            AllocateStatic(size) => {
                self.memory.registers[POINTER_REG] =
                    Types::Pointer(self.memory.allocate_obj(size), PointerTypes::Object);
                self.next_line();
            }
            Reallocate(size_reg) => {
                if let Types::Pointer(u_size, ptr_type) = self.memory.registers[POINTER_REG] {
                    match ptr_type {
                        PointerTypes::Object => {
                            if let Types::Uint(new_size) = self.memory.registers[size_reg] {
                                self.memory.resize_obj(u_size, new_size);
                            } else {
                                return self.panic_rt(ErrTypes::WrongTypeOperation(
                                    self.memory.registers[size_reg],
                                    self.code.data[self.code.ptr],
                                ));
                            }
                        }
                        PointerTypes::String => {
                            if let Types::Uint(new_size) = self.memory.registers[size_reg] {
                                let append_len = new_size - self.memory.strings.pool[u_size].len();
                                let str = "\0".repeat(append_len);
                                self.memory.strings.pool[u_size].push_str(&str);
                            } else {
                                return self.panic_rt(ErrTypes::WrongTypeOperation(
                                    self.memory.registers[size_reg],
                                    self.code.data[self.code.ptr],
                                ));
                            }
                        }
                        _ => {
                            return self.panic_rt(ErrTypes::WrongTypeOperation(
                                self.memory.registers[POINTER_REG],
                                self.code.data[self.code.ptr],
                            ))
                        }
                    }
                } else {
                    return self.panic_rt(ErrTypes::WrongTypeOperation(
                        self.memory.registers[POINTER_REG],
                        self.code.data[self.code.ptr],
                    ));
                }
                self.next_line();
            }
            Deallocate => {
                if let Types::Pointer(u_size, ptr_type) = self.memory.registers[POINTER_REG] {
                    match ptr_type {
                        PointerTypes::Object => {
                            self.memory.deallocate_obj(u_size);
                        }
                        PointerTypes::String => {
                            self.memory.deallocate_string(u_size);
                        }
                        _ => {
                            return self.panic_rt(ErrTypes::WrongTypeOperation(
                                self.memory.registers[POINTER_REG],
                                self.code.data[self.code.ptr],
                            ))
                        }
                    }
                } else {
                    return self.panic_rt(ErrTypes::WrongTypeOperation(
                        self.memory.registers[POINTER_REG],
                        self.code.data[self.code.ptr],
                    ));
                }
                self.next_line();
            }
            Sweep => {
                self.memory.gc_sweep();
                self.next_line();
            }
            SweepUnoptimized => {
                self.memory.gc_sweep_unoptimized();
                self.next_line();
            }
            Goto(pos) => {
                self.code.ptr = pos;
            }
            Jump(pos) => {
                self.memory.stack.call_stack[self.memory.stack.ptr].code_ptr = self.code.ptr;
                self.code.ptr = pos;
            }
            GotoPtr => {
                if let Types::Function(u_size) = self.memory.registers[CODE_PTR_REG] {
                    self.code.ptr = self.memory.fun_table[u_size].loc;
                } else {
                    return self.panic_rt(ErrTypes::InvalidType(
                        self.memory.registers[CODE_PTR_REG],
                        Types::Function(0),
                    ));
                }
            }
            DynReserve(reg_id) => {
                if let Types::Function(u_size) = self.memory.registers[reg_id] {
                    if let Some((size, pointers_len)) = self.memory.fun_table[u_size].stack_size {
                        let end = self.stack_end() + size;
                        self.memory.stack.ptr += 1;
                        if self.memory.stack.ptr >= self.memory.stack.call_stack.len() {
                            if self.memory.stack.ptr > self.memory.stack.call_stack.len() {
                                loop {
                                    println!(
                                        "Samik mel pravdu, ale tohle stejne nikdy neuvidis ;p"
                                    );
                                }
                            }
                            return self.panic_rt(ErrTypes::StackOverflow);
                        }
                        self.memory.stack.call_stack[self.memory.stack.ptr].end = end;
                        self.memory.stack.call_stack[self.memory.stack.ptr].pointers_len =
                            pointers_len;
                        if end > self.memory.stack.data.len() {
                            self.memory.stack.data.resize(end, Types::Null);
                        }
                    }
                } else {
                    return self.panic_rt(ErrTypes::InvalidType(
                        self.memory.registers[CODE_PTR_REG],
                        Types::Function(0),
                    ));
                }
                self.next_line();
            }
            DynArgument(id_reg, arg_num, value_reg) => {
                if let Types::Function(u_size) = self.memory.registers[id_reg] {
                    let where_to = &self.memory.fun_table[u_size].params[arg_num];
                    match where_to {
                        MemoryLoc::Stack(offset) => {
                            let end = self.stack_end();
                            self.memory.stack.data[end - offset] = self.memory.registers[value_reg];
                        }
                        MemoryLoc::Register(reg) => {
                            self.memory.registers[*reg] = self.memory.registers[value_reg];
                        }
                    }
                } else {
                    return self.panic_rt(ErrTypes::InvalidType(
                        self.memory.registers[CODE_PTR_REG],
                        Types::Function(0),
                    ));
                }
                self.next_line();
            }
            Branch(pos1, pos2) => {
                if let Types::Bool(bool) = self.memory.registers[GENERAL_REG1] {
                    self.code.ptr = if bool { pos1 } else { pos2 };
                } else {
                    return self.panic_rt(ErrTypes::WrongTypeOperation(
                        self.memory.registers[GENERAL_REG1],
                        self.code.data[self.code.ptr],
                    ));
                }
            }
            Return => {
                self.memory.stack.ptr -= 1;
                self.code.ptr = self.memory.stack.call_stack[self.memory.stack.ptr].code_ptr;
                self.next_line();
            }
            Back => {
                self.code.ptr = self.memory.stack.call_stack[self.memory.stack.ptr].code_ptr;
                self.next_line();
            }
            Unfreeze => {
                for i in 0..FREEZED_REG_SIZE {
                    self.memory.registers[i] =
                        self.memory.stack.call_stack[self.memory.stack.ptr].reg_freeze[i]
                }
                self.next_line();
            }
            ReserveStack(size, pointers_len) => {
                let end = self.stack_end() + size;
                self.memory.stack.ptr += 1;
                if self.memory.stack.ptr >= self.memory.stack.call_stack.len() {
                    if self.memory.stack.ptr > self.memory.stack.call_stack.len() {
                        loop {
                            println!("Samik mel pravdu, ale tohle stejne nikdy neuvidis ;p");
                        }
                    }
                    return self.panic_rt(ErrTypes::StackOverflow);
                }
                self.memory.stack.call_stack[self.memory.stack.ptr].end = end;
                self.memory.stack.call_stack[self.memory.stack.ptr].pointers_len = pointers_len;
                if end > self.memory.stack.data.len() {
                    self.memory.stack.data.resize(end, Types::Null);
                }
                self.next_line();
            }
            Freeze => {
                self.memory.stack.call_stack[self.memory.stack.ptr]
                    .reg_freeze
                    .clone_from_slice(&self.memory.registers[..6]);
                self.next_line();
            }
            Swap(reg1, reg2) => {
                let temp = self.memory.registers[reg1];
                self.memory.registers[reg1] = self.memory.registers[reg2];
                self.memory.registers[reg2] = temp;
                self.next_line();
            }
            Move(reg1, reg2) => {
                self.memory.registers[reg2] = self.memory.registers[reg1];
                self.next_line();
            }
            Add(r1, r2, res) => {
                match self.memory.registers[r1] {
                    Types::Int(num1) => operation!(Int, add, num1, r1, r2, res),
                    Types::Float(num1) => operation!(Float, add, num1, r1, r2, res),
                    Types::Uint(num1) => operation!(Uint, add, num1, r1, r2, res),
                    Types::Char(char1) => {
                        if let Types::Char(char2) = self.memory.registers[r2] {
                            self.memory.registers[res] =
                                Types::Char((char1 as u8 + char2 as u8) as char);
                        } else {
                            return self.panic_rt(ErrTypes::WrongTypeOperation(
                                self.memory.registers[r2],
                                self.code.data[self.code.ptr],
                            ));
                        }
                    }
                    _ => {
                        return self.panic_rt(ErrTypes::WrongTypeOperation(
                            self.memory.registers[r1],
                            self.code.data[self.code.ptr],
                        ));
                    }
                }
                self.next_line();
            }
            Sub(r1, r2, res) => {
                match self.memory.registers[r1] {
                    Types::Int(num1) => operation!(Int, sub, num1, r1, r2, res),
                    Types::Float(num1) => operation!(Float, sub, num1, r1, r2, res),
                    Types::Uint(num1) => operation!(Uint, sub, num1, r1, r2, res),
                    Types::Char(char1) => {
                        if let Types::Char(char2) = self.memory.registers[r2] {
                            self.memory.registers[res] =
                                Types::Char((char1 as u8 - char2 as u8) as char);
                        } else {
                            return self.panic_rt(ErrTypes::WrongTypeOperation(
                                self.memory.registers[r2],
                                self.code.data[self.code.ptr],
                            ));
                        }
                    }
                    _ => {
                        return self.panic_rt(ErrTypes::WrongTypeOperation(
                            self.memory.registers[r1],
                            self.code.data[self.code.ptr],
                        ));
                    }
                }
                self.next_line();
            }
            Mul(r1, r2, res) => {
                match self.memory.registers[r1] {
                    Types::Int(num1) => operation!(Int, mul, num1, r1, r2, res),
                    Types::Float(num1) => operation!(Float, mul, num1, r1, r2, res),
                    Types::Uint(num1) => operation!(Uint, mul, num1, r1, r2, res),
                    Types::Char(char1) => {
                        if let Types::Char(char2) = self.memory.registers[r2] {
                            self.memory.registers[res] =
                                Types::Char((char1 as u8 * char2 as u8) as char);
                        } else {
                            return self.panic_rt(ErrTypes::WrongTypeOperation(
                                self.memory.registers[r2],
                                self.code.data[self.code.ptr],
                            ));
                        }
                    }
                    _ => {
                        return self.panic_rt(ErrTypes::WrongTypeOperation(
                            self.memory.registers[r1],
                            self.code.data[self.code.ptr],
                        ));
                    }
                }
                self.next_line();
            }
            Div(r1, r2, res) => {
                match self.memory.registers[r1] {
                    Types::Int(num1) => operation!(Int, div, num1, r1, r2, res),
                    Types::Float(num1) => operation!(Float, div, num1, r1, r2, res),
                    Types::Uint(num1) => operation!(Uint, div, num1, r1, r2, res),
                    Types::Char(char1) => {
                        if let Types::Char(char2) = self.memory.registers[r2] {
                            self.memory.registers[res] =
                                Types::Char((char1 as u8 / char2 as u8) as char);
                        } else {
                            return self.panic_rt(ErrTypes::WrongTypeOperation(
                                self.memory.registers[r2],
                                self.code.data[self.code.ptr],
                            ));
                        }
                    }
                    _ => {
                        return self.panic_rt(ErrTypes::WrongTypeOperation(
                            self.memory.registers[r1],
                            self.code.data[self.code.ptr],
                        ));
                    }
                }
                self.next_line();
            }
            Mod(r1, r2, res) => {
                match self.memory.registers[r1] {
                    Types::Int(num1) => operation!(Int, %, num1, r1, r2, res),
                    Types::Float(num1) => operation!(Float, %, num1, r1, r2, res),
                    Types::Uint(num1) => operation!(Uint, %, num1, r1, r2, res),
                    Types::Char(char1) => {
                        if let Types::Char(char2) = self.memory.registers[r2] {
                            self.memory.registers[res] =
                                Types::Char((char1 as u8 % char2 as u8) as char);
                        } else {
                            return self.panic_rt(ErrTypes::WrongTypeOperation(
                                self.memory.registers[r2],
                                self.code.data[self.code.ptr],
                            ));
                        }
                    }
                    _ => {
                        return self.panic_rt(ErrTypes::WrongTypeOperation(
                            self.memory.registers[r1],
                            self.code.data[self.code.ptr],
                        ));
                    }
                }
                self.next_line();
            }
            Equ(r1, r2, res) => {
                match self.memory.registers[r1] {
                    Types::Int(num1) => operation!(Int, eq, num1, bool, r1, r2, res),
                    Types::Float(num1) => operation!(Float, eq, num1, bool, r1, r2, res),
                    Types::Uint(num1) => operation!(Uint, eq, num1, bool, r1, r2, res),
                    Types::Pointer(num1, _) => operation!(ptr, eq, num1, bool, r1, r2, res),
                    Types::Bool(var1) => operation!(Bool, eq, var1, bool, r1, r2, res),
                    Types::Char(char1) => operation!(Char, eq, char1, bool, r1, r2, res),
                    Types::Null => match self.memory.registers[r2] {
                        Types::Null => self.memory.registers[res] = Types::Bool(true),
                        _ => self.memory.registers[res] = Types::Bool(false),
                    },
                    _ => {
                        return self.panic_rt(ErrTypes::WrongTypeOperation(
                            self.memory.registers[r1],
                            self.code.data[self.code.ptr],
                        ));
                    }
                }
                self.next_line();
            }
            Grt(r1, r2, res) => {
                match self.memory.registers[r1] {
                    Types::Int(num1) => operation!(Int, gt, num1, bool, r1, r2, res),
                    Types::Float(num1) => operation!(Float, gt, num1, bool, r1, r2, res),
                    Types::Uint(num1) => operation!(Uint, gt, num1, bool, r1, r2, res),
                    Types::Char(char1) => operation!(Char, gt, char1, bool, r1, r2, res),
                    _ => {
                        return self.panic_rt(ErrTypes::WrongTypeOperation(
                            self.memory.registers[r1],
                            self.code.data[self.code.ptr],
                        ));
                    }
                }
                self.next_line();
            }
            Less(r1, r2, res) => {
                match self.memory.registers[r1] {
                    Types::Int(num1) => operation!(Int, lt, num1, bool, r1, r2, res),
                    Types::Float(num1) => operation!(Float, lt, num1, bool, r1, r2, res),
                    Types::Uint(num1) => operation!(Uint, lt, num1, bool, r1, r2, res),
                    Types::Char(char1) => operation!(Char, lt, char1, bool, r1, r2, res),
                    _ => {
                        return self.panic_rt(ErrTypes::WrongTypeOperation(
                            self.memory.registers[r1],
                            self.code.data[self.code.ptr],
                        ));
                    }
                }
                self.next_line();
            }
            And(r1, r2, res) => {
                match self.memory.registers[r1] {
                    Types::Bool(var1) => {
                        if let Types::Bool(var2) = self.memory.registers[r2] {
                            self.memory.registers[res] = Types::Bool(var1 && var2)
                        } else {
                            return self.panic_rt(ErrTypes::CrossTypeOperation(
                                self.memory.registers[r1],
                                self.memory.registers[r2],
                                self.code.data[self.code.ptr],
                            ));
                        }
                    }
                    _ => {
                        return self.panic_rt(ErrTypes::WrongTypeOperation(
                            self.memory.registers[r1],
                            self.code.data[self.code.ptr],
                        ));
                    }
                }
                self.next_line();
            }
            Or(r1, r2, res) => {
                match self.memory.registers[r1] {
                    Types::Bool(var1) => {
                        if let Types::Bool(var2) = self.memory.registers[r2] {
                            self.memory.registers[res] = Types::Bool(var1 || var2)
                        } else {
                            return self.panic_rt(ErrTypes::CrossTypeOperation(
                                self.memory.registers[r1],
                                self.memory.registers[r2],
                                self.code.data[self.code.ptr],
                            ));
                        }
                    }
                    _ => {
                        return self.panic_rt(ErrTypes::WrongTypeOperation(
                            self.memory.registers[r1],
                            self.code.data[self.code.ptr],
                        ));
                    }
                }
                self.next_line();
            }
            Not(r1, res) => {
                match self.memory.registers[r1] {
                    Types::Bool(var) => self.memory.registers[res] = Types::Bool(!var),
                    _ => {
                        return self.panic_rt(ErrTypes::WrongTypeOperation(
                            self.memory.registers[r1],
                            self.code.data[self.code.ptr],
                        ));
                    }
                }
                self.next_line();
            }
            Cal(lib, fun_id) => {
                match self.libs[lib].clone()(
                    self,
                    fun_id
                ) {
                    Ok(value) => {
                        if let Types::Void = value {
                        } else {
                            self.memory.registers[RETURN_REG] = value;
                        }
                    }
                    Err(err) => {
                        return self.panic_rt(err);
                    }
                }
                self.next_line();
            }
            DynMethod(obj, trt, method) => {
                if let Types::NonPrimitive(id) = self.memory.registers[obj] {
                    if let Some(method) = self.memory.non_primitives[id]
                        .methods
                        .get(&trt)
                        .unwrap()
                        .get(method)
                    {
                        self.memory.stack.call_stack[self.memory.stack.ptr].code_ptr =
                            self.code.ptr;
                        self.code.ptr = *method;
                    } else {
                        return self.panic_rt(ErrTypes::MethodNotFound);
                    }
                } else {
                    return self.panic_rt(ErrTypes::WrongTypeOperation(
                        self.memory.registers[obj],
                        self.code.data[self.code.ptr],
                    ));
                }
            }
            End => {
                return false;
            }
            Debug(reg) => {
                println!("{:+}", self.memory.registers[reg]);
                self.next_line();
            }
            Len(reg) => {
                if let Types::NonPrimitive(kind) = self.memory.registers[reg] {
                    self.memory.registers[reg] = Types::Uint(self.memory.non_primitives[kind].len)
                } else if let Types::Pointer(u_size, kind) = self.memory.registers[reg] {
                    match kind {
                        PointerTypes::Object => {
                            self.memory.registers[reg] =
                                Types::Uint(self.memory.heap.data[u_size].len())
                        }
                        PointerTypes::String => {
                            self.memory.registers[reg] =
                                Types::Uint(self.memory.strings.pool[u_size].len())
                        }
                        _ => {
                            return self.panic_rt(ErrTypes::WrongTypeOperation(
                                self.memory.registers[reg],
                                self.code.data[self.code.ptr],
                            ));
                        }
                    }
                } else {
                    return self.panic_rt(ErrTypes::InvalidType(
                        self.memory.registers[reg],
                        Types::NonPrimitive(0),
                    ));
                }
                self.next_line()
            }
            // todo optimize
            CopyRange(original, new, len) => {
                let new_ptr = if let Types::Pointer(u_size, kind) = self.memory.registers[new] {
                    (u_size, kind)
                } else {
                    return self.panic_rt(ErrTypes::WrongTypeOperation(
                        self.memory.registers[new],
                        CopyRange(0, 0, 0),
                    ));
                };
                if let Types::Pointer(u_size, kind) = self.memory.registers[original] {
                    for i in 0..len {
                        let value = match kind {
                            PointerTypes::Object => self.memory.heap.data[u_size][i],
                            PointerTypes::String => Types::Pointer(u_size, PointerTypes::String),
                            PointerTypes::Stack => self.memory.stack.data[u_size + i],
                            PointerTypes::Heap(idx) => self.memory.heap.data[u_size][i + idx],
                            PointerTypes::Char(idx) => Types::Char(
                                self.memory.strings.pool[u_size]
                                    .chars()
                                    .nth(i + idx)
                                    .unwrap(),
                            ),
                            PointerTypes::UserData => {
                                return self.panic_rt(ErrTypes::CannotReadUserdata);
                            }
                        };
                        match new_ptr.1 {
                            PointerTypes::Object => {
                                self.memory.heap.data[new_ptr.0][i] = value;
                            }
                            PointerTypes::String => {
                                self.memory.strings.pool[new_ptr.0]
                                    .replace_range(i..i + 1, &value.get_char().to_string());
                            }
                            PointerTypes::Stack => {
                                self.memory.stack.data[new_ptr.0 + i] = value;
                            }
                            PointerTypes::Heap(idx) => {
                                self.memory.heap.data[new_ptr.0][idx + i] = value;
                            }
                            PointerTypes::Char(idx) => {
                                self.memory.strings.pool[new_ptr.0].replace_range(
                                    idx + i..idx + i + 1,
                                    &value.get_char().to_string(),
                                );
                            }
                            PointerTypes::UserData => {
                                return self.panic_rt(ErrTypes::CannotReadUserdata);
                            }
                        }
                    }
                } else {
                    return self.panic_rt(ErrTypes::WrongTypeOperation(
                        self.memory.registers[original],
                        CopyRange(0, 0, 0),
                    ));
                }
            }
            // TODO: optimize
            // - dont match on each iteration
            FillRange(val, len) => {
                let value = self.memory.registers[val];
                let len = if let Types::Uint(len) = self.memory.registers[len] {
                    len
                } else {
                    return self.panic_rt(ErrTypes::WrongTypeOperation(
                        self.memory.registers[len],
                        FillRange(0, 0),
                    ));
                };
                if let Types::Pointer(u_size, kind) = self.memory.registers[POINTER_REG] {
                    for i in 0..len {
                        match kind {
                            PointerTypes::Object => {
                                self.memory.heap.data[u_size][i] = value;
                            }
                            PointerTypes::Stack => {
                                self.memory.stack.data[u_size + i] = value;
                            }
                            PointerTypes::Heap(idx) => {
                                self.memory.heap.data[u_size][i + idx] = value;
                            }
                            PointerTypes::Char(_idx) => {
                                todo!()
                            }
                            PointerTypes::String => {
                                todo!()
                            }
                            PointerTypes::UserData => {
                                return self.panic_rt(ErrTypes::CannotReadUserdata);
                            }
                        }
                    }
                }
                self.next_line();
            }
            Type(reg1, reg2) => {
                use std::mem::discriminant;
                self.memory.registers[reg2] = Types::Bool(
                    discriminant(&self.memory.registers[reg1])
                        == discriminant(&self.memory.registers[reg2]),
                );
                self.next_line();
            }
            NonPrimitiveType(np_reg, id) => {
                if let Types::NonPrimitive(id_dyn) = self.memory.registers[np_reg] {
                    self.memory.registers[GENERAL_REG3] = Types::Bool(id == id_dyn);
                } else {
                    return self.panic_rt(ErrTypes::Expected(
                        Types::NonPrimitive(0),
                        self.memory.registers[np_reg],
                    ));
                }
            }
            Cast(reg1, ttype) => {
                match Self::cast(&mut self.memory.registers, reg1, ttype) {
                    Ok(value) => {
                        self.memory.registers[reg1] = value;
                    }
                    Err(err) => return self.panic_rt(err),
                }
                self.next_line();
            }
            Break(code) => {
                self.break_code = Some(code);
                return false;
            }
            Catch => {
                if let Err(err) = self.catches.push(runtime_types::Catch {
                    code_ptr: self.code.ptr,
                    id: None,
                    cs_ptr: self.memory.stack.ptr,
                }) {
                    return self.panic_rt(err);
                }
                self.next_line()
            }
            CatchId(id) => {
                if let Err(err) = self.catches.push(runtime_types::Catch {
                    code_ptr: self.code.ptr,
                    id: Some(id),
                    cs_ptr: self.memory.stack.ptr,
                }) {
                    return self.panic_rt(err);
                }
                self.next_line()
            }
            DeleteCatch(n) => {
                self.catches.truncate(self.catches.cache.len() - n);
                self.next_line()
            }
            /*StrCpy(reg) => {
                if let Types::Pointer(u_size, PointerTypes::String) = self.memory.registers[reg] {
                    self.memory.registers[POINTER_REG] =
                        Types::Pointer(self.memory.strings.copy(u_size), PointerTypes::String);
                } else {
                    return self.panic_rt(ErrTypes::Expected(
                        Types::Pointer(0, PointerTypes::String),
                        self.memory.registers[reg],
                    ));
                }
                self.next_line();
            }*/
            StrNew => {
                self.memory.registers[POINTER_REG] =
                    Types::Pointer(self.memory.strings.new(), PointerTypes::String);
                self.next_line();
            }
            IntoStr(reg) => {
                self.memory.registers[POINTER_REG] = Types::Pointer(
                    self.memory
                        .strings
                        .from_str(&self.memory.registers[reg].to_str(&self.memory)),
                    PointerTypes::String,
                );
                self.next_line();
            }
            Panic => {
                self.enter_panic();
                self.next_line();
            }
            Neg(reg) => {
                match self.memory.registers[reg] {
                    Types::Int(num) => self.memory.registers[reg] = Types::Int(-num),
                    Types::Float(num) => self.memory.registers[reg] = Types::Float(-num),
                    _ => {
                        return self.panic_rt(ErrTypes::WrongTypeOperation(
                            self.memory.registers[reg],
                            self.code.data[self.code.ptr],
                        ));
                    }
                }
                self.next_line();
            }
            WriteArg(pos, reg) => {
                self.memory.args.data[self.memory.args.ptr][pos] = self.memory.registers[reg];
                self.next_line();
            }
            ReadArg(pos, reg) => {
                self.memory.registers[reg] = self.memory.args.data[self.memory.args.ptr][pos];
                self.next_line();
            }
            OpenArgs => {
                self.memory.args.ptr += 1;
                self.next_line();
            }
            CloseArgs => {
                self.memory.args.ptr -= 1;
                self.next_line();
            }
            Null => {
                if let Types::Null = self.memory.registers[GENERAL_REG1] {
                    self.memory.registers[GENERAL_REG1] = Types::Bool(true);
                } else {
                    self.memory.registers[GENERAL_REG1] = Types::Bool(false);
                }
                self.next_line();
            }
        }
        return true;
    }
    fn stack_end(&self) -> usize {
        self.memory.stack.call_stack[self.memory.stack.ptr].end
    }
    fn next_line(&mut self) {
        self.code.ptr += 1;
    }
    fn cast(registers: &mut Registers, reg1: usize, reg2: usize) -> Result<Types, ErrTypes> {
        match registers[reg1] {
            Types::Bool(bol) => match registers[reg2] {
                Types::Int(_) => {
                    return if bol {
                        Ok(Types::Int(1))
                    } else {
                        Ok(Types::Int(0))
                    }
                }
                Types::Float(_) => {
                    return if bol {
                        Ok(Types::Float(1f64))
                    } else {
                        Ok(Types::Float(0f64))
                    }
                }
                Types::Uint(_) => {
                    return if bol {
                        Ok(Types::Uint(1))
                    } else {
                        Ok(Types::Uint(0))
                    }
                }
                Types::Char(_) => {
                    return if bol {
                        Ok(Types::Char('1'))
                    } else {
                        Ok(Types::Char('0'))
                    }
                }
                _ => return Err(ErrTypes::ImplicitCast(registers[reg1], registers[reg2])),
            },
            Types::Int(num) => match registers[reg2] {
                Types::Float(_) => return Ok(Types::Float(num as f64)),
                Types::Uint(_) => return Ok(Types::Uint(num as usize)),
                Types::Char(_) => return Ok(Types::Char(num as u8 as char)),
                Types::Bool(_) => {
                    return if num == 0 {
                        Ok(Types::Bool(false))
                    } else {
                        Ok(Types::Bool(true))
                    }
                }
                _ => return Err(ErrTypes::ImplicitCast(registers[reg1], registers[reg2])),
            },
            Types::Float(num) => match registers[reg2] {
                Types::Int(_) => return Ok(Types::Int(num as i64)),
                Types::Uint(_) => return Ok(Types::Uint(num as usize)),
                Types::Char(_) => return Ok(Types::Char(num as u8 as char)),
                Types::Bool(_) => {
                    return if num == 0f64 {
                        Ok(Types::Bool(false))
                    } else {
                        Ok(Types::Bool(true))
                    }
                }
                _ => return Err(ErrTypes::ImplicitCast(registers[reg1], registers[reg2])),
            },
            Types::Uint(num) => match registers[reg2] {
                Types::Int(_) => return Ok(Types::Int(num as i64)),
                Types::Float(_) => return Ok(Types::Float(num as f64)),
                Types::Char(_) => return Ok(Types::Char(num as u8 as char)),
                Types::Bool(_) => {
                    return if num == 0 {
                        Ok(Types::Bool(false))
                    } else {
                        Ok(Types::Bool(true))
                    }
                }
                _ => return Err(ErrTypes::ImplicitCast(registers[reg1], registers[reg2])),
            },
            Types::Char(char) => match registers[reg2] {
                Types::Int(_) => return Ok(Types::Int(char as i64)),
                Types::Float(_) => return Ok(Types::Float(char as u8 as f64)),
                Types::Uint(_) => return Ok(Types::Uint(char as usize)),
                Types::Bool(_) => {
                    return if char == '\0' {
                        Ok(Types::Bool(false))
                    } else {
                        Ok(Types::Bool(true))
                    }
                }
                _ => return Err(ErrTypes::ImplicitCast(registers[reg1], registers[reg2])),
            },
            _ => return Err(ErrTypes::ImplicitCast(registers[reg1], registers[reg2])),
        }
    }
    fn panic_rt(&mut self, kind: ErrTypes) -> bool {
        if self.enter_panic() {
            return true;
        }
        self.break_code = Some(self.code.ptr);
        println!("{}", get_message(&kind, Some((self.code.ptr, 0))));
        self.exit_code = ExitCodes::Internal(kind);
        false
    }
    /// This function is called when an exception is thrown. It will search for a catch block
    /// that matches the exception type. If it finds one, it will set the code pointer to the
    /// catch block and set the stack pointer to the catch block's stack pointer. If it doesn't
    /// find one, it will set the exit code to exception and return false. (indicating that the VM should exit)
    fn enter_panic(&mut self) -> bool {
        let mut i = self.catches.catches_ptr;
        loop {
            if i == 0 {
                self.exit_code = ExitCodes::Exception;
                return false;
            }
            i -= 1;
            if let Some(n) = self.catches.cache[i].id {
                if let Types::NonPrimitive(e_type) = self.memory.registers[RETURN_REG] {
                    if n == e_type {
                        self.code.ptr = self.catches.cache[i].code_ptr;
                        self.memory.stack.ptr = self.catches.cache[i].cs_ptr;
                    }
                }
                break;
            } else {
                self.code.ptr = self.catches.cache[i].code_ptr;
                self.memory.stack.ptr = self.catches.cache[i].cs_ptr;
                break;
            }
        }
        self.catches.truncate(i);
        true
    }
    pub fn size(&self) -> usize {
        self.memory.size()
            + std::mem::size_of_val(&self.break_code)
            + std::mem::size_of_val(&self.catches.cache)
            + std::mem::size_of_val(&self.catches.catches_ptr)
            + std::mem::size_of_val(&self.code)
            + std::mem::size_of_val(&self.exit_code)
            + std::mem::size_of_val(&self.libs)
    }
    pub fn instruction_debug(&self) -> String {
        const DEF: String = String::new();
        let mut prepend = format!("{}. {:?} ", self.code.ptr, self.code.data[self.code.ptr]);
        let val = match &self.code.data[self.code.ptr] {
            Instructions::Debug(_) => DEF,
            Instructions::Write(_, register) => format!(
                "data: {}",
                self.memory.registers[*register].to_str(&self.memory)
            ),
            Instructions::Read(stack, _) => format!(
                "data: {}",
                self.memory.stack.data
                    [self.memory.stack.call_stack[self.memory.stack.ptr].end - *stack]
                    .to_str(&self.memory)
            ),
            Instructions::WritePtr(reg) => {
                format!("data: {}", self.memory.registers[*reg].to_str(&self.memory))
            }
            Instructions::ReadPtr(_) => DEF,
            Instructions::ReadConst(pos, _) => format!(
                "data: {}",
                self.memory.stack.data[*pos].to_str(&self.memory)
            ),
            Instructions::Ptr(_) => DEF,
            Instructions::Index(_) => DEF,
            Instructions::Allocate(_) => DEF,
            Instructions::Reallocate(_) => DEF,
            Instructions::Deallocate => DEF,
            Instructions::Goto(instr) => format!("destination: {}", self.code.data[*instr]),
            Instructions::GotoPtr => DEF,
            Instructions::Branch(a, b) => format!(
                "destination: {} or {}",
                self.code.data[*a], self.code.data[*b]
            ),
            Instructions::Return => format!(
                "destination: {}. {}; value: {}",
                self.memory.stack.call_stack[self.memory.stack.ptr].code_ptr,
                self.code.data[self.memory.stack.call_stack[self.memory.stack.ptr].code_ptr],
                self.memory.registers[RETURN_REG].to_str(&self.memory)
            ),
            Instructions::Unfreeze => DEF,
            Instructions::ReserveStack(_, _) => DEF,
            Instructions::Swap(a, b) => format!(
                "registers: {} and {}",
                self.memory.registers[*a].to_str(&self.memory),
                self.memory.registers[*b].to_str(&self.memory)
            ),
            Instructions::Add(a, b, _) => format!(
                "registers: {} and {}",
                self.memory.registers[*a].to_str(&self.memory),
                self.memory.registers[*b].to_str(&self.memory)
            ),
            Instructions::Sub(a, b, _) => format!(
                "registers: {} and {}",
                self.memory.registers[*a].to_str(&self.memory),
                self.memory.registers[*b].to_str(&self.memory)
            ),
            Instructions::Mul(a, b, _) => format!(
                "registers: {} and {}",
                self.memory.registers[*a].to_str(&self.memory),
                self.memory.registers[*b].to_str(&self.memory)
            ),
            Instructions::Div(a, b, _) => format!(
                "registers: {} and {}",
                self.memory.registers[*a].to_str(&self.memory),
                self.memory.registers[*b].to_str(&self.memory)
            ),
            Instructions::Mod(a, b, _) => format!(
                "registers: {} and {}",
                self.memory.registers[*a].to_str(&self.memory),
                self.memory.registers[*b].to_str(&self.memory)
            ),
            Instructions::Equ(a, b, _) => format!(
                "registers: {} and {}",
                self.memory.registers[*a].to_str(&self.memory),
                self.memory.registers[*b].to_str(&self.memory)
            ),
            Instructions::Grt(a, b, _) => format!(
                "registers: {} and {}",
                self.memory.registers[*a].to_str(&self.memory),
                self.memory.registers[*b].to_str(&self.memory)
            ),
            Instructions::Less(a, b, _) => format!(
                "registers: {} and {}",
                self.memory.registers[*a].to_str(&self.memory),
                self.memory.registers[*b].to_str(&self.memory)
            ),
            Instructions::And(a, b, _) => format!(
                "registers: {} and {}",
                self.memory.registers[*a].to_str(&self.memory),
                self.memory.registers[*b].to_str(&self.memory)
            ),
            Instructions::Or(a, b, _) => format!(
                "registers: {} and {}",
                self.memory.registers[*a].to_str(&self.memory),
                self.memory.registers[*b].to_str(&self.memory)
            ),
            Instructions::Not(a, _) => format!(
                "register: {}",
                self.memory.registers[*a].to_str(&self.memory)
            ),
            Instructions::Cal(_, _) => DEF,
            Instructions::End => DEF,
            Instructions::Cast(a, b) => format!(
                "registers: {} and {}",
                self.memory.registers[*a].to_str(&self.memory),
                self.memory.registers[*b].to_str(&self.memory)
            ),
            Instructions::Len(_) => DEF,
            Instructions::Type(_, _) => DEF,
            Instructions::Jump(dest) => {
                format!("destination: {}. {}", *dest, self.code.data[*dest])
            }
            Instructions::Freeze => DEF,
            Instructions::Back => format!(
                "destination: {}",
                self.code.data[self.memory.stack.call_stack[self.memory.stack.ptr].code_ptr]
            ),
            Instructions::Move(val, _) => format!(
                "register: {}",
                self.memory.registers[*val].to_str(&self.memory)
            ),
            Instructions::Sweep => DEF,
            Instructions::SweepUnoptimized => DEF,
            Instructions::AllocateStatic(_) => DEF,
            Instructions::IndexStatic(_) => DEF,
            Instructions::FillRange(val, _) => format!(
                "register: {}",
                self.memory.registers[*val].to_str(&self.memory)
            ),
            Instructions::CopyRange(_, _, _) => DEF,
            Instructions::Break(_) => DEF,
            Instructions::DynMethod(_, _, _) => DEF,
            Instructions::Panic => DEF,
            Instructions::Catch => DEF,
            Instructions::CatchId(_) => DEF,
            Instructions::DeleteCatch(_) => DEF,
            Instructions::NonPrimitiveType(reg, id) => format!(
                "register: {}; name: {}",
                self.memory.registers[*reg].to_str(&self.memory),
                self.memory.non_primitives[*id].name
            ),
            Instructions::StrNew => DEF,
            Instructions::IntoStr(val) => format!(
                "register: {}",
                self.memory.registers[*val].to_str(&self.memory)
            ),
            Instructions::DynReserve(reg) => format!(
                "reserved: {}",
                self.memory.registers[*reg].to_str(&self.memory)
            ),
            Instructions::DynArgument(_, _, _) => DEF,
            Instructions::Neg(reg) => format!(
                "register: {}",
                self.memory.registers[*reg].to_str(&self.memory)
            ),
            Instructions::WriteArg(pos, reg) => format!(
                "data: {} to {}",
                self.memory.registers[*reg].to_str(&self.memory),
                *pos
            ),
            Instructions::ReadArg(pos, reg) => format!(
                "data: {} from {}",
                self.memory.stack.data[*pos].to_str(&self.memory),
                *pos
            ),
            Instructions::OpenArgs => DEF,
            Instructions::CloseArgs => {
                format!("args: {:?}", self.memory.args.data[self.memory.args.ptr])
            }
            Instructions::NullCheck => format!("register0: {}", self.memory.registers[0]),
        };
        prepend.push_str(&val);
        prepend
    }
}
#[allow(unused)]
pub mod runtime_types {
    pub const CALL_STACK_SIZE: usize = 256;
    pub const MAX_ARGS: usize = 15;
    pub const FREEZED_REG_SIZE: usize = 6;
    pub type Registers = [Types; REGISTER_SIZE];
    pub const REGISTER_SIZE: usize = 9;
    pub const GENERAL_REG1: usize = 0;
    pub const GENERAL_REG2: usize = 1;
    pub const GENERAL_REG3: usize = 2;
    pub const MEMORY_REG1: usize = 3;
    pub const MEMORY_REG2: usize = 4;
    pub const ARGS_REG: usize = 5;
    pub const POINTER_REG: usize = REGISTER_SIZE - 3;
    pub const RETURN_REG: usize = REGISTER_SIZE - 2;
    pub const CODE_PTR_REG: usize = REGISTER_SIZE - 1;
    /// context for a single thread of execution (may include multiple threads in future updates)
    /// this is the main struct that holds all the data for the runtime
    pub struct Context {
        pub memory: Memory,
        pub code: Code,
        pub break_code: Option<usize>,
        pub catches: Catches,
        pub exit_code: ExitCodes,
        pub(crate) libs: Libs,
    }
    pub struct Memory {
        pub stack: Stack,
        pub args: Args,
        pub registers: Registers,
        pub heap: Heap,
        pub strings: Strings,
        pub non_primitives: Vec<NonPrimitiveType>,
        pub gc: GarbageCollector,
        pub fun_table: Vec<FunSpec>,
        pub runtime_args: Vec<String>,
        pub user_data: UserDataContainer,
    }
    pub struct UserDataContainer {
        pub data: Vec<Box<dyn user_data::UserData>>,
        pub garbage: Vec<usize>,
    }
    pub struct GarbageCollector {
        pub sweeps_count: usize,
        pub memory_swept: usize,
        pub disabled: bool,
        pub sweep_threshold: usize,
    }
    impl Memory {
        // allocator starts here
        pub fn allocate_obj(&mut self, size: usize) -> usize {
            let mut data = Vec::new();
            data.resize(size, Types::Null);
            if let Some(idx) = self.heap.garbage.pop() {
                self.heap.data[idx] = data;
                return idx;
            }
            self.heap.data.push(data);
            self.heap.data.len() - 1
        }
        pub fn deallocate_obj(&mut self, idx: usize) -> bool {
            if idx >= self.heap.data.len() {
                return false;
            }
            if idx == self.heap.data.len() - 1 {
                self.heap.data.pop();
                // get largest index of non garbage obj using last obj and truncate
                let last = self.last_obj();
                self.heap.data.truncate(last);
                return true;
            }
            self.heap.garbage.push(idx);
            self.heap.data[idx].clear();
            true
        }
        pub fn last_string(&mut self) -> usize {
            if self.strings.pool.is_empty() {
                return 0;
            }
            // find first string that is garbage and following strings are garbage and dont remove any strings from garbage
            let mut i = self.strings.pool.len() - 1;
            loop {
                // if string is garbage and all strings after it are garbage then return i + 1
                if self.strings.garbage.iter().any(|e| *e == i) {
                    if i == 0 {
                        return 0;
                    }
                    i -= 1;
                } else {
                    return i + 1;
                }
            }
        }
        pub fn last_obj(&mut self) -> usize {
            if self.heap.data.is_empty() {
                return 0;
            }
            // find first object that is garbage and following objects are garbage and dont remove any objects from garbage
            let mut i = self.heap.data.len() - 1;
            loop {
                // if object is garbage and all objects after it are garbage then return i + 1
                if self.heap.garbage.iter().any(|e| *e == i) {
                    if i == 0 {
                        return 0;
                    }
                    i -= 1;
                } else {
                    return i + 1;
                }
            }
        }
        pub fn last_ud(&mut self) -> usize {
            if self.user_data.data.is_empty() {
                return 0;
            }
            // find first object that is garbage and following objects are garbage and dont remove any objects from garbage
            let mut i = self.user_data.data.len() - 1;
            loop {
                // if object is garbage and all objects after it are garbage then return i + 1
                if self.user_data.garbage.iter().any(|e| *e == i) {
                    if i == 0 {
                        return 0;
                    }
                    i -= 1;
                } else {
                    return i + 1;
                }
            }
        }
        pub fn deallocate_string(&mut self, idx: usize) -> bool {
            if idx >= self.strings.pool.len() {
                return false;
            }
            if idx == self.strings.pool.len() - 1 {
                self.strings.pool.pop();
                // get largest index of non garbage string and truncate
                let last = self.last_string();
                self.strings.pool.truncate(last);
                return true;
            }
            self.strings.garbage.push(idx);
            self.strings.pool[idx].clear();
            true
        }
        /// resizes the object
        pub fn resize_obj(&mut self, heap_idx: usize, new_size: usize) {
            self.heap.data[heap_idx].resize(new_size, Types::Null)
        }
        /// resizes the object relative to its current len
        pub fn grow_obj(&mut self, heap_idx: usize, new_size: i64) {
            let size = self.heap.data[heap_idx].len();
            self.heap.data[heap_idx].resize((size as i64 + new_size) as usize, Types::Null)
        }
        pub fn verify_obj(&mut self, heap_idx: usize, id: usize) -> bool {
            let obj = &self.heap.data[heap_idx][0];
            if let Types::NonPrimitive(_id) = obj {
                *_id == id
            } else {
                false
            }
        }
        /// returns the len of object ignoring the header
        pub fn obj_len(&self, heap_idx: usize) -> usize {
            self.heap.data[heap_idx].len()
        }
        pub fn size_of(&self, val: &Types) -> usize {
            match val {
                Types::NonPrimitive(id) => self.non_primitives[*id].len,
                _ => 1,
            }
        }
        pub fn write_ptr(
            &mut self,
            loc: usize,
            kind: &PointerTypes,
            value: &Types,
        ) -> Result<(), ErrTypes> {
            match kind {
                PointerTypes::Stack => {
                    self.stack.data[loc] = *value;
                }
                PointerTypes::Heap(idx) => {
                    self.heap.data[loc][*idx] = *value;
                }
                PointerTypes::Object => {
                    self.heap.data[loc][0] = *value;
                }
                PointerTypes::String => {
                    if let Types::Pointer(dest, PointerTypes::String) = value {
                        self.strings.copy_from(loc, *dest)
                    } else {
                        return Err(ErrTypes::Expected(
                            Types::Pointer(0, PointerTypes::String),
                            *value,
                        ));
                    }
                }
                PointerTypes::Char(idx) => {
                    if let Types::Char(chr) = value {
                        self.strings.pool[loc].replace_range(*idx..=*idx + 1, &chr.to_string());
                    } else {
                        return Err(ErrTypes::Expected(Types::Char('a'), *value));
                    }
                }
                PointerTypes::UserData => {
                    return Err(ErrTypes::CannotReadUserdata);
                }
            }
            Ok(())
        }
        /// writes value to pointer with index
        ///
        /// btw this needs rewrite asap TODO: rewrite this pls pslspplsplssplsp
        pub fn write_idx(
            &mut self,
            loc: usize,
            kind: &mut PointerTypes,
            idx: i64,
            value: &Types,
        ) -> Result<(), ErrTypes> {
            // TODO: my god pls rewrite this disgusting piece of shit
            let mut ptr = Types::Pointer(loc, *kind);
            ptr.index(idx);
            self.write_ptr(ptr.ptr_loc(), &ptr.kind(), value)
        }
        /// returns value the pointer points at (or void)
        pub fn index(&self, ptr: Types, idx: usize) -> Types {
            if let Types::Pointer(u_size, kind) = ptr {
                return match kind {
                    PointerTypes::Stack => self.stack.data[u_size + idx].clone(),
                    PointerTypes::Heap(loc) => self.heap.data[u_size][loc + idx].clone(),
                    PointerTypes::Object => self.heap.data[u_size][idx].clone(),
                    PointerTypes::String => Types::Char(
                        self.strings.pool[u_size]
                            .chars()
                            .next()
                            .expect("tried to get char that may not exist"),
                    ),
                    PointerTypes::Char(loc) => Types::Char(
                        self.strings.pool[u_size]
                            .chars()
                            .nth(loc + idx)
                            .expect("tried to get char that may not exist"),
                    ),
                    PointerTypes::UserData => Types::Void,
                };
            }
            Types::Void
        }
        pub fn get(&self, ptr: Types) -> Types {
            if let Types::Pointer(u_size, kind) = ptr {
                return match kind {
                    PointerTypes::Stack => self.stack.data[u_size].clone(),
                    PointerTypes::Heap(loc) => self.heap.data[u_size][loc].clone(),
                    PointerTypes::Object => self.heap.data[u_size][0].clone(),
                    PointerTypes::String => ptr,
                    PointerTypes::Char(c) => Types::Char(
                        self.strings.pool[u_size]
                            .chars()
                            .nth(c)
                            .expect("tried to get char that may not exist"),
                    ),
                    PointerTypes::UserData => Types::Void,
                };
            }
            Types::Void
        }
        pub fn args(&self) -> &[Types; MAX_ARGS] {
            let ptr = self.args.ptr;
            &self.args.data[ptr]
        }
        /// GC
        pub fn gc_sweep(&mut self) {
            if self.gc.disabled {
                return;
            }
            let marked = self.gc_mark();
            self.gc_sweep_marked(marked);
        }
        pub fn gc_sweep_unoptimized(&mut self) {
            if self.gc.disabled {
                return;
            }
            let marked = self.gc_mark_unoptimized();
            self.gc_sweep_marked(marked);
        }
        pub fn gc_sweep_marked(&mut self, marked: (Vec<bool>, Vec<bool>, Vec<bool>)) {
            self.gc_sweep_marked_obj(marked.0);
            self.gc_sweep_marked_string(marked.1);
            self.gc_sweep_marked_ud(marked.2);
            let last = self.last_string();
            self.strings.pool.truncate(last);
            let last = self.last_obj();
            self.heap.data.truncate(last);
            let last = self.last_ud();
            self.user_data.data.truncate(last);
        }
        pub fn gc_sweep_marked_obj(&mut self, marked: Vec<bool>) {
            if let Some(idx) = marked.iter().rposition(|x| !*x) {
                self.gc.memory_swept += std::mem::size_of_val(&self.heap.data[idx..]);
                self.heap.data.truncate(idx + 1);
            } else {
                self.gc.memory_swept += std::mem::size_of_val(&self.heap.data[..]);
                self.heap.data.clear();
                return;
            }
            for (i, mark) in marked.iter().enumerate() {
                if i >= self.heap.data.len() {
                    return;
                }
                if *mark {
                    self.gc.memory_swept += std::mem::size_of_val(&self.heap.data[i]);
                    self.heap.data[i].clear();
                    //self.heap.data[i].shrink_to(0);
                    if !self.heap.garbage.contains(&i) {
                        self.heap.garbage.push(i);
                    }
                }
            }
        }
        pub fn gc_sweep_marked_string(&mut self, marked: Vec<bool>) {
            // find first string that is garbage and following strings are garbage and then remove them from garbage
            if let Some(idx) = marked.iter().rposition(|x| !*x) {
                self.gc.memory_swept += std::mem::size_of_val(&self.strings.pool[idx..]);
                self.strings.pool.truncate(idx + 1);
            } else {
                self.gc.memory_swept += std::mem::size_of_val(&self.strings.pool);
                self.strings.pool.clear();
                return;
            }
            // remove all strings that are marked
            for (i, mark) in marked.iter().enumerate() {
                if i >= self.strings.pool.len() {
                    continue;
                }
                if *mark {
                    self.gc.memory_swept += std::mem::size_of_val(&self.strings.pool[i]);
                    self.strings.pool[i].clear();
                    if !self.strings.garbage.contains(&i) {
                        self.strings.garbage.push(i);
                    }
                }
            }
        }
        pub fn gc_sweep_marked_ud(&mut self, marked: Vec<bool>) {
            // find first ud that is garbage and following uds are garbage and then remove them from garbage
            if let Some(idx) = marked.iter().rposition(|x| !*x) {
                self.gc.memory_swept += std::mem::size_of_val(&self.user_data.data[idx..]);
                self.user_data.data.truncate(idx + 1);
            } else {
                self.gc.memory_swept += std::mem::size_of_val(&self.user_data.data);
                self.user_data.data.clear();
                return;
            }
            // remove all uds that are marked
            for (i, mark) in marked.iter().enumerate() {
                if i >= self.user_data.data.len() {
                    continue;
                }
                if *mark {
                    self.gc.memory_swept += std::mem::size_of_val(&self.user_data.data[i]);
                    self.user_data.data[i].cleanup();
                    if !self.user_data.garbage.contains(&i) {
                        self.user_data.garbage.push(i);
                    }
                }
            }
        }
        pub fn gc_mark_unoptimized(&mut self) -> (Vec<bool>, Vec<bool>, Vec<bool>) {
            let mut marked_obj = vec![true; self.heap.data.len()];
            let mut marked_str = vec![true; self.strings.pool.len()];
            for i in 0..self.strings.static_strings {
                marked_str[i] = false;
            }
            let mut marked_ud = vec![true; self.user_data.data.len()];
            self.gc_mark_registers(&mut marked_obj, &mut marked_str, &mut marked_ud);
            self.gc_mark_range(
                (0, self.stack.data.len()),
                &mut marked_obj,
                &mut marked_str,
                &mut marked_ud,
            );
            self.gc_mark_args(&mut marked_obj, &mut marked_str, &mut marked_ud);
            (marked_obj, marked_str, marked_ud)
        }
        pub fn gc_mark(&mut self) -> (Vec<bool>, Vec<bool>, Vec<bool>) {
            let mut call_stack_idx = 1;
            let mut marked = vec![false; self.heap.data.len()];
            let mut marked_str = vec![false; self.strings.pool.len()];
            let mut marked_ud = vec![false; self.user_data.data.len()];
            self.gc_mark_registers(&mut marked, &mut marked_str, &mut marked_ud);
            self.gc_mark_args(&mut marked, &mut marked_str, &mut marked_ud);
            while call_stack_idx <= self.stack.ptr {
                let cs = self.stack.call_stack[call_stack_idx];
                let prev_cs = self.stack.call_stack[call_stack_idx - 1];
                self.gc_mark_range(
                    (prev_cs.end, prev_cs.end + cs.pointers_len),
                    &mut marked,
                    &mut marked_str,
                    &mut marked_ud,
                );
                call_stack_idx += 1;
            }
            (marked, marked_str, marked_ud)
        }
        pub fn gc_mark_obj(
            &mut self,
            obj_idx: usize,
            marked: &mut Vec<bool>,
            marked_str: &mut Vec<bool>,
            marked_ud: &mut Vec<bool>
        ) {
            if marked.len() == 0 {
                return;
            }
            if !marked[obj_idx] {
                return;
            }
            marked[obj_idx] = false;
            for idx in 0..self.heap.data[obj_idx].len() {
                let member = self.heap.data[obj_idx][idx];
                if let Types::Pointer(u_size, PointerTypes::Object) = member {
                    self.gc_mark_obj(u_size, marked, marked_str, marked_ud);
                } else if let Types::Pointer(u_size, PointerTypes::Heap(_)) = member {
                    self.gc_mark_obj(u_size, marked, marked_str, marked_ud);
                } else if let Types::Pointer(u_size, PointerTypes::String) = member {
                    self.gc_mark_string(u_size, marked_str)
                } else if let Types::Pointer(u_size, PointerTypes::UserData) = member {
                    self.gc_mark_ud(u_size, marked_ud);
                }
            }
        }
        pub fn gc_mark_string(&mut self, str_idx: usize, marked: &mut Vec<bool>) {
            marked[str_idx] = false;
        }
        pub fn gc_mark_ud(&mut self, ud_idx: usize, marked: &mut Vec<bool>) {
            use user_data::GcMethod;
            match self.user_data.data[ud_idx].gc_method() {
                GcMethod::None => {}
                GcMethod::Gc => {
                    marked[ud_idx] = false;
                }
                GcMethod::Own => {
                    self.user_data.data[ud_idx].cleanup();
                    marked[ud_idx] = false;
                }
            }
        }
        pub fn gc_mark_range(
            &mut self,
            range: (usize, usize),
            marked_obj: &mut Vec<bool>,
            marked_string: &mut Vec<bool>,
            marked_ud: &mut Vec<bool>,
        ) {
            for idx in range.0..range.1 {
                match self.stack.data[idx] {
                    Types::Pointer(u_size, PointerTypes::Heap(_)) => {
                        self.gc_mark_obj(u_size, marked_obj, marked_string, marked_ud);
                    }
                    Types::Pointer(u_size, PointerTypes::Object) => {
                        self.gc_mark_obj(u_size, marked_obj, marked_string, marked_ud);
                    }
                    Types::Pointer(u_size, PointerTypes::String) => {
                        self.gc_mark_string(u_size, marked_string);
                    }
                    Types::Pointer(u_size, PointerTypes::Char(_)) => {
                        self.gc_mark_string(u_size, marked_string);
                    }
                    Types::Pointer(u_size, PointerTypes::UserData) => {
                        self.gc_mark_ud(u_size, marked_ud);
                    }
                    _ => {}
                }
            }
        }
        pub fn gc_mark_registers(
            &mut self,
            marked: &mut Vec<bool>,
            marked_str: &mut Vec<bool>,
            marked_ud: &mut Vec<bool>,
        ) {
            for reg in self.registers {
                match reg {
                    Types::Pointer(u_size, PointerTypes::Heap(_)) => {
                        self.gc_mark_obj(u_size, marked, marked_str, marked_ud);
                    }
                    Types::Pointer(u_size, PointerTypes::Object) => {
                        self.gc_mark_obj(u_size, marked, marked_str, marked_ud);
                    }
                    Types::Pointer(u_size, PointerTypes::String) => {
                        self.gc_mark_string(u_size, marked_str);
                    }
                    Types::Pointer(u_size, PointerTypes::Char(_)) => {
                        self.gc_mark_string(u_size, marked_str);
                    }
                    Types::Pointer(u_size, PointerTypes::UserData) => {
                        self.gc_mark_ud(u_size, marked_ud);
                    }
                    _ => {}
                }
            }
        }
        pub fn gc_mark_args(
            &mut self,
            marked: &mut Vec<bool>,
            marked_str: &mut Vec<bool>,
            marked_ud: &mut Vec<bool>,
        ) {
            for i in 0..self.args.ptr {
                for arg in self.args.data[i] {
                    match arg {
                        Types::Pointer(u_size, PointerTypes::Heap(_)) => {
                            self.gc_mark_obj(u_size, marked, marked_str, marked_ud);
                        }
                        Types::Pointer(u_size, PointerTypes::Object) => {
                            self.gc_mark_obj(u_size, marked, marked_str, marked_ud);
                        }
                        Types::Pointer(u_size, PointerTypes::String) => {
                            self.gc_mark_string(u_size, marked_str);
                        }
                        Types::Pointer(u_size, PointerTypes::Char(_)) => {
                            self.gc_mark_string(u_size, marked_str);
                        }
                        Types::Pointer(u_size, PointerTypes::UserData) => {
                            self.gc_mark_ud(u_size, marked_ud);
                        }
                        _ => {}
                    }
                }
            
            }
        }
        /// return the size of memory in bytes
        pub fn size(&self) -> usize {
            std::mem::size_of_val(&self.heap)
                + std::mem::size_of_val(&self.heap.data)
                + std::mem::size_of_val(&self.stack)
                + std::mem::size_of_val(&self.stack.data)
                + std::mem::size_of_val(&self.stack.call_stack)
                + std::mem::size_of_val(&self.stack.ptr)
                + std::mem::size_of_val(&self.strings.pool)
                + std::mem::size_of_val(&self.registers)
                + std::mem::size_of_val(&self.non_primitives)
        }
    }
    impl UserDataContainer {
        pub fn new() -> Self {
            Self {
                data: Vec::new(),
                garbage: Vec::new(),
            }
        }
        pub fn push(&mut self, data: Box<dyn user_data::UserData>) -> usize {
            if let Some(idx) = self.garbage.pop() {
                self.data[idx] = data;
                idx
            } else {
                self.data.push(data);
                self.data.len() - 1
            }
        }
        pub fn remove(&mut self, idx: usize) -> Option<Box<dyn user_data::UserData>> {
            if idx >= self.data.len() {
                return None;
            }
            self.garbage.push(idx);
            Some(std::mem::replace(
                &mut self.data[idx],
                Box::new(user_data::Null),
            ))
        }
    }
    pub type Lib = fn(ctx: &mut Context, id: usize) -> Result<Types, ErrTypes>;
    pub type Libs = Vec<Lib>;
    pub struct Stack {
        pub data: Vec<Types>,
        pub ptr: usize,
        pub call_stack: [CallStack; CALL_STACK_SIZE],
    }
    pub struct Args {
        pub data: [[Types; MAX_ARGS]; CALL_STACK_SIZE],
        pub ptr: usize,
    }
    pub struct Heap {
        pub data: Vec<Vec<Types>>,
        pub garbage: Vec<usize>,
    }
    pub struct Strings {
        pub pool: Vec<String>,
        pub garbage: Vec<usize>,
        /// number of strings that will never be deleted
        pub static_strings: usize,
    }
    #[allow(unused)]
    impl Strings {
        /// Creates a new empty string and returns the location of the string
        pub fn new(&mut self) -> usize {
            // either push a new string or occupy a deleted string
            if let Some(loc) = self.garbage.pop() {
                self.pool[loc] = String::new();
                loc
            } else {
                self.pool.push(String::new());
                self.pool.len() - 1
            }
        }
        /// Creates a new string from a &str and returns the location of the string
        pub fn from_str(&mut self, str: &str) -> usize {
            // either push a new string or occupy a deleted string
            if let Some(loc) = self.garbage.pop() {
                self.pool[loc] = str.to_string();
                loc
            } else {
                self.pool.push(str.to_string());
                self.pool.len() - 1
            }
        }
        /// Takes ownership of a String and returns the location of the string in vm
        pub fn from_string(&mut self, str: String) -> usize {
            // either push a new string or occupy a deleted string
            if let Some(loc) = self.garbage.pop() {
                self.pool[loc] = str;
                loc
            } else {
                self.pool.push(str);
                self.pool.len() - 1
            }
        }
        /// Copies a string from one location to a new location and returns the new location
        pub fn copy(&mut self, loc: usize) -> usize {
            // either push a new string or occupy a deleted string
            if let Some(new_loc) = self.garbage.pop() {
                self.pool[new_loc] = self.pool[loc].clone();
                new_loc
            } else {
                self.pool.push(self.pool[loc].clone());
                self.pool.len() - 1
            }
        }
        /// Copies a string from one location to another location.
        pub fn copy_from(&mut self, orig: usize, dest: usize) {
            self.pool[dest] = self.pool[orig].clone()
        }
        pub fn concat(&mut self, left: usize, right: usize) -> usize {
            let mut temp = self.pool[left].clone();
            temp.push_str(&self.pool[right]);
            self.from_str(&temp)
        }
        pub fn push_string_array(&mut self, arr: Vec<&str>) -> Vec<usize> {
            let mut temp = Vec::with_capacity(arr.len());
            for str in arr {
                temp.push(self.from_str(str));
            }
            temp
        }
        pub fn to_string(&self, loc: usize) -> String {
            self.pool[loc].clone()
        }
        pub fn to_str(&self, loc: usize) -> &str {
            &self.pool[loc]
        }
    }
    pub struct Code {
        pub data: Vec<Instructions>,
        pub ptr: usize,
        pub entry_point: usize,
    }
    /// a structure that holds information about a function
    /// this is used to call dynamic methods
    #[derive(Debug)]
    pub struct FunSpec {
        pub name: String,
        pub params: Vec<MemoryLoc>,
        /// size, pointers_len
        pub stack_size: Option<(usize, usize)>,
        pub loc: usize,
    }
    /// a structure that holds information about the placement of a value in memory
    #[derive(Debug)]
    pub enum MemoryLoc {
        Stack(usize),
        Register(usize),
    }
    /// a structure exposed to the linked libraries
    pub struct PublicData<'a> {
        pub memory: &'a mut Memory,
        /// Instructions
        ///
        /// It is read only, for security reasons
        pub code: &'a Code,
        pub break_code: &'a mut Option<usize>,
        pub exit_code: &'a mut ExitCodes,
    }
    #[derive(Debug, Copy, Clone)]
    pub struct Catches {
        pub catches_ptr: usize,
        pub cache: [Catch; CALL_STACK_SIZE],
    }
    impl Catches {
        /// pushes a new catch to the stack
        pub fn push(&mut self, catch: Catch) -> Result<(), ErrTypes> {
            if self.catches_ptr == CALL_STACK_SIZE {
                return Err(ErrTypes::CatchOwerflow);
            }
            self.catches_ptr += 1;
            self.cache[self.catches_ptr] = catch;
            Ok(())
        }
        /// pops the last catch from the stack
        pub fn pop(&mut self) {
            self.catches_ptr -= 1;
        }
        /// truncates the stack to a given size
        pub fn truncate(&mut self, n: usize) {
            self.catches_ptr = n;
        }
    }
    /// a structure that holds information about the location of each catch
    #[derive(Debug, Copy, Clone)]
    pub struct Catch {
        /// location of the catch
        ///
        /// does not need to be exact, for example
        /// if there is more than one catch in a function,
        /// then the location of the catch can be the end of those catches
        ///
        /// therefor the location of the catch is the location of the last catch
        pub code_ptr: usize,
        /// location of the stack the catch was created in
        pub cs_ptr: usize,
        /// Describes the type of the catch
        ///
        /// If the catch is a catch all, then the type is None
        pub id: Option<usize>,
    }
    /// indicates why program exited
    #[derive(Debug, Clone)]
    pub enum ExitCodes {
        /// program ended
        End,
        /// program run into user defined break and is expected to continue in the future
        Break(usize),
        /// an exception was thrown but never caught
        Exception,
        /// unrecoverable error occured (if you believe this is not meant to happen, contact me)
        Internal(runtime_error::ErrTypes),
        /// program got signal to break from the outside
        OuterBreak,
    }
    const TYPES_SIZE: usize = std::mem::size_of::<Types>();
    #[derive(Clone, Copy, Debug)]
    pub enum Types {
        Int(i64),
        Float(f64),
        Uint(usize),
        Char(char),
        Bool(bool),
        Pointer(usize, PointerTypes),
        Function(usize),
        // null represents an empty value
        Null,
        // void represents a value that is not meant to be used
        Void,
        /// header for non-primitive types
        /// ID
        NonPrimitive(usize),
    }
    impl Types {
        /// may panic, so use this only if you are 100% certain that you got a character
        pub fn get_char(&self) -> char {
            if let Types::Char(chr) = self {
                return *chr;
            }
            unreachable!("tried to get char from non-char type")
        }
        pub fn to_str(&self, mem: &Memory) -> String {
            match *self {
                Types::Bool(b) => b.to_string(),
                Types::Char(c) => c.to_string(),
                Types::Int(i) => i.to_string(),
                Types::Float(f) => f.to_string(),
                Types::Null => "null".to_string(),
                Types::NonPrimitive(kind) => kind.to_string(),
                Types::Uint(val) => val.to_string(),
                Types::Pointer(u_size, val) => match val {
                    PointerTypes::Char(chr) => format!("Ptr<String, {}, {}>", u_size, chr),
                    PointerTypes::Heap(idx) => format!("Ptr<Heap, {}, {}>", u_size, idx),
                    PointerTypes::Object => format!("Ptr<Heap, {}>", u_size),
                    PointerTypes::Stack => format!("Ptr<Stack, {}>", u_size),
                    PointerTypes::String => format!("Ptr<String, {}>", u_size),
                    PointerTypes::UserData => format!(
                        "Ptr<UserData, {}, {}>",
                        u_size,
                        mem.user_data.data[u_size].label()
                    ),
                },
                Types::Function(val) => mem.fun_table[val].name.to_string(),
                Types::Void => "void".to_string(),
            }
        }
        /// returns the first index of pointer
        pub fn ptr_loc(&self) -> usize {
            match *self {
                Types::Pointer(loc, _) => loc,
                _ => 0,
            }
        }
        /// returns the second index of pointer or 0
        pub fn ptr_idx(&self) -> usize {
            match *self {
                Types::Pointer(_, PointerTypes::Heap(idx)) => idx,
                Types::Pointer(_, PointerTypes::Char(idx)) => idx,
                _ => 0,
            }
        }
        /// returns kind of pointer
        pub fn kind(&self) -> PointerTypes {
            match *self {
                Types::Pointer(_, kind) => kind,
                _ => PointerTypes::Stack,
            }
        }
        /// advances the pointer by index
        pub fn index(&mut self, idx: i64) {
            match *self {
                Types::Pointer(ref mut loc, ref mut kind) => {
                    *loc = (*loc as i64 + idx) as usize;
                    match kind {
                        PointerTypes::Heap(ref mut i) => *i = (*i as i64 + idx) as usize,
                        PointerTypes::Char(ref mut i) => *i = (*i as i64 + idx) as usize,
                        _ => (),
                    }
                }
                _ => (),
            }
        }
    }
    #[derive(Clone, Copy, Debug)]
    pub enum NonPrimitiveTypes {
        Array,
        Struct,
    }
    /// a structure used to store non-primitive types
    /// this is basically a VTable
    #[derive(Debug, Clone)]
    pub struct NonPrimitiveType {
        pub name: String,
        pub kind: NonPrimitiveTypes,
        /// number of values in the type (including header)
        pub len: usize,
        pub pointers: usize,
        /// first index is trait id, second is method id
        pub methods: HashMap<usize, Vec<usize>>,
    }
    use std::{clone, collections::HashMap, fmt, hash::Hash, ops::Index, rc::Rc, sync::Arc};

    use crate::user_data::{self, UserData};

    use super::{
        runtime_error::{self, ErrTypes},
    };
    impl fmt::Display for Types {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            if f.alternate() {
                match *self {
                    Types::Bool(_) => write!(f, "Bool"),
                    Types::Char(_) => write!(f, "Char"),
                    Types::Function(_) => write!(f, "CodePointer"),
                    Types::Float(_) => write!(f, "Float"),
                    Types::Int(_) => write!(f, "Int"),
                    Types::Null => write!(f, "Null"),
                    Types::Pointer(_, _) => write!(f, "Pointer"),
                    Types::Uint(_) => write!(f, "Uint"),
                    Types::NonPrimitive(_) => write!(f, "Non-primitive"),
                    Types::Void => write!(f, "Void"),
                }
            } else if f.sign_plus() {
                match *self {
                    Types::Bool(bol) => {
                        write!(f, "Bool<{bol}>")
                    }
                    Types::Char(char) => write!(f, "Char<{char}>"),
                    Types::Function(loc) => write!(f, "CodePointer<{loc}>"),
                    Types::Float(num) => write!(f, "Float<{num}>"),
                    Types::Int(num) => write!(f, "Int<{num}>"),
                    Types::Null => write!(f, "Null"),
                    Types::Pointer(loc, kind) => write!(f, "Pointer<{loc}, {kind}>"),
                    Types::Uint(num) => write!(f, "Uint<{num}>"),
                    Types::NonPrimitive(id) => write!(f, "Non-primitive<{id}>"),
                    Types::Void => write!(f, "Void"),
                }
            } else {
                match *self {
                    Types::Bool(bol) => write!(f, "{bol}"),
                    Types::Char(char) => write!(f, "{char}"),
                    Types::Function(loc) => write!(f, "{loc}"),
                    Types::Float(num) => write!(f, "{num}"),
                    Types::Int(num) => write!(f, "{num}"),
                    Types::Null => write!(f, "Null"),
                    Types::Pointer(loc, _) => write!(f, "{loc}"),
                    Types::Uint(num) => write!(f, "{num}"),
                    Types::NonPrimitive(id) => write!(f, "{id}"),
                    Types::Void => write!(f, "Void"),
                }
            }
        }
    }
    impl fmt::Display for NonPrimitiveTypes {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                NonPrimitiveTypes::Array => write!(f, "Array"),
                NonPrimitiveTypes::Struct => write!(f, "Struct"),
            }
        }
    }
    /// runtime
    #[derive(Clone, Copy, Debug)]
    pub enum PointerTypes {
        /// location on stack
        ///
        /// expires out of scope
        Stack,
        /// object
        /// needs to be transformed into heap pointer
        /// with index(usize)
        ///
        /// never expires, GC may change value
        Object,
        /// location on heap
        ///
        /// may expire any time
        Heap(usize),
        /// String
        ///
        /// location in string pool
        /// never expires
        String,
        /// char
        ///
        /// location and index in string pool
        /// may expire any time
        Char(usize),
        /// user data
        ///
        /// location in user data pool
        /// may expire any time if parent library drops it
        UserData,
    }
    impl PointerTypes {
        pub fn is_object(&self) -> bool {
            match *self {
                PointerTypes::Object => true,
                _ => false,
            }
        }
        pub fn index(&mut self, idx: i64) {
            match *self {
                PointerTypes::Heap(ref mut i) => *i = (*i as i64 + idx) as usize,
                PointerTypes::Char(ref mut i) => *i = (*i as i64 + idx) as usize,
                PointerTypes::Object => *self = PointerTypes::Heap(idx as usize),
                _ => (),
            }
        }
        pub fn value(&self) -> usize {
            match *self {
                PointerTypes::Heap(i) => i,
                PointerTypes::Char(i) => i,
                _ => 0,
            }
        }
    }
    impl fmt::Display for PointerTypes {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                PointerTypes::Heap(n) => write!(f, "Heap({n})"),
                PointerTypes::Object => write!(f, "Object"),
                PointerTypes::Stack => write!(f, "Stack"),
                PointerTypes::String => write!(f, "String"),
                PointerTypes::Char(n) => write!(f, "Stack({n})"),
                PointerTypes::UserData => write!(f, "UserData"),
            }
        }
    }
    /// complete list of runtime instructions
    #[allow(unused)]
    #[derive(Clone, Copy, Debug)]
    pub enum Instructions {
        /// Debug: reg | prints value of reg(<reg>)
        Debug(usize),
        /// Write: stack_offset reg | moves value from reg(0) to stack(stack_end - <stack_offset>)
        Write(usize, usize),
        /// Read: stack_offset reg | reads value from stack(stack_end - <stack_offset>) to its reg(<reg>)
        Read(usize, usize),
        /// WritePointer: value_reg | moves value from reg(<value_reg>) to stack(<pointer>)
        WritePtr(usize),
        /// ReadPointer: reg | reads value from reg(pointer_reg) to its reg(<reg>)
        ReadPtr(usize),
        /// ReadConstant: stack_pos reg | reads value from stack(<stack_pos>) to its reg(<reg>)
        ReadConst(usize, usize),
        /// Pointer: stack_pos | stores pointer to stack(stack_end - <stack_offset>) in reg(0)
        Ptr(usize),
        /// Index: idx | gets pointer from reg(<pointer>) repairs it and adds reg(<idx>)
        Index(usize),
        /// Allocate: size_reg pointers_len | reserves <size> on heap and stores location in registers(pointer_reg)
        Allocate(usize),
        /// Reallocate: size_reg | resizes heap(<reg>) for <size>; additional space is filled with null
        Reallocate(usize),
        /// Free: | frees heap(<reg>)
        Deallocate,
        /// Goto: pos | moves code_pointer to <pos>
        Goto(usize),
        /// GotoCodePtr: pos_reg | moves code pointer to reg(<reg>)
        GotoPtr,
        /// Branch: pos1 pos2 | if reg(0), goto <pos1> else goto <pos2>
        Branch(usize, usize),
        /// Return: | moves code_pointer to the last position in callstack and moves callstack back
        Return,
        /// Unfreeze | returns registers to their last freezed state
        Unfreeze,
        /// Reserve: size ptrs | reserves <size> on stack and advances callstack, also saves number of pointers for faster memory sweeps
        ReserveStack(usize, usize),
        /// Swap: reg1 reg2   | swaps <reg1> and <reg2>
        Swap(usize, usize),
        /// Add | reg(0) is set to the result of operation: reg(0) + reg(1)
        Add(usize, usize, usize),
        /// Subtract | reg(0) is set to the result of operation: reg(0) - reg(1)
        Sub(usize, usize, usize),
        /// Multiply | reg(0) is set to the result of operation: reg(0) * reg(1)
        Mul(usize, usize, usize),
        /// Divide | reg(0) is set to the result of operation: reg(0) / reg(1)
        Div(usize, usize, usize),
        /// Modulus | reg(0) is set to the result of operation: reg(0) % reg(1)
        Mod(usize, usize, usize),
        /// Equals | reg(0) is set to the result of operation: reg(0) = reg(1)
        Equ(usize, usize, usize),
        /// Greater than | reg(0) is set to the result of operation: reg(0) > reg(1)
        Grt(usize, usize, usize),
        /// Less than | reg(0) is set to the result of operation: reg(0) < reg(1)
        Less(usize, usize, usize),
        /// And | reg(0) is set to the result of operation: reg(0) & reg(1)
        And(usize, usize, usize),
        /// Or | reg(0) is set to the result of operation: reg(0) | reg(1)
        Or(usize, usize, usize),
        /// Not | reg(0) is set to the result of operation: !reg(0)
        Not(usize, usize),
        /// Call | calls external <procedure>(program state, <args>) written in rust (for syscalls etc..)
        Cal(usize, usize),
        /// End              | terminates program
        End,
        //TODO: add to compiler
        /// Cast: reg1 reg2 | casts value of reg1 to the type of reg2 and stores in reg1
        Cast(usize, usize),
        /// Length: reg | sets reg to Uint(size of an object)
        Len(usize),
        /// Type: val type | sets reg(type) to bool(typeof(val) == typeof(type))
        Type(usize, usize),
        /// Jump: pos | moves code_pointer to <pos> and saves current code ptr
        Jump(usize),
        /// Freeze | freezes registers on callstack
        Freeze,
        /// Back | returns to last code ptr
        Back,
        /// Move: reg1 reg2 | moves value of reg1 to reg2
        Move(usize, usize),
        /// Sweep | sweeps memory, deallocating all unaccesable objects
        Sweep,
        /// Sweep unoptimized | sweeps memory, deallocating all unaccesable objects, this instruction is here only to help me test GC since it doesnt require any code structure
        SweepUnoptimized,
        /// Allocate static: size | allocates new object with size known at compile time and returns pointer to reg(ptr)
        AllocateStatic(usize),
        /// Index static: index | indexing operation where index is known at compile time (generally for structures but can be also used for arrays or single values on heap)
        IndexStatic(usize),
        /// Fill range: val_reg len | takes pointer at reg(POINTER_REG) as a starting point and fills len to the right with value on reg(value_reg)
        FillRange(usize, usize),
        /// Copy range: original_ptr new_ptr len | copies range starting at reg(original_ptr) with size len to reg(new_ptr)
        CopyRange(usize, usize, usize),
        /// Break: code | program exits with a break code, indicating that it should be resumed at some point
        Break(usize),
        /// Method: struct trait method | takes struct and calls method on it, assuming it implements trait  
        DynMethod(usize, usize, usize),
        /// Panic | program enters panic mode, returning from all stacks until exception is caught
        Panic,
        /// Catch | catches an error and returns program to normal mode, cached if read in normal mode
        Catch,
        /// Catch ID: id | same as normal catch but responds only to exception with same id
        CatchId(usize),
        /// Delete catch | deletes specified amount of catches from stack
        DeleteCatch(usize),
        /// Non-primitive type: np_reg ID | compares reg(np_reg).id assuming it belongs to Non-primitive type with ID
        NonPrimitiveType(usize, usize),
        /// String new | creates new string and stores pointer in reg(POINTER_REGISTER)
        StrNew,
        /// Into string: val_reg | converts value on reg(value_reg) to string and stores pointer in reg(POINTER_REG)
        IntoStr(usize),
        /// Reserve dynamic: id_reg | prepares memory for anonymous function call (may allocate size on stack) based on fun_table(id_reg).stack_size
        DynReserve(usize),
        /// Argument dynamic: id_reg arg_num value_reg | pushes arguments to destination(stack or registers) based on fun_table(id_reg).params
        DynArgument(usize, usize, usize),
        /// Negate: reg | negates value of reg
        Neg(usize),
        /// Write argument: arg_num value_reg | writes value of reg(value_reg) to argument on stack
        WriteArg(usize, usize),
        /// Read argument: arg_num reg | reads value of argument on stack to reg(reg)
        ReadArg(usize, usize),
        /// Open args: | opens arguments for function call
        OpenArgs,
        /// Close args: | closes arguments of function call
        CloseArgs,
        /// Null | sets checks if reg(0) is null and sets reg(0) to bool
        NullCheck,
    }
    impl fmt::Display for Instructions {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let str = match *self {
                Instructions::Add(_, _, _) => "Addition",
                Instructions::Allocate(_) => "Allocation",
                Instructions::AllocateStatic(_) => "Allocation",
                Instructions::And(_, _, _) => "And",
                Instructions::Branch(_, _) => "Branch",
                Instructions::Cal(_, _) => "Call",
                Instructions::Debug(_) => "Debug",
                Instructions::Div(_, _, _) => "Division",
                Instructions::End => "End",
                Instructions::Equ(_, _, _) => "Equality",
                Instructions::Goto(_) => "GoTo",
                Instructions::GotoPtr => "GoToDyn",
                Instructions::Grt(_, _, _) => "Greater",
                Instructions::Index(_) => "Indexing",
                Instructions::IndexStatic(_) => "Indexing",
                Instructions::Less(_, _, _) => "Lesser",
                Instructions::Mod(_, _, _) => "Modulus",
                Instructions::Swap(_, _) => "Swap",
                Instructions::Mul(_, _, _) => "Multiplication",
                Instructions::Not(_, _) => "Not",
                Instructions::Or(_, _, _) => "Or",
                Instructions::Ptr(_) => "StackPointer",
                Instructions::Reallocate(_) => "Reallocation",
                Instructions::Unfreeze => "Unfreeze",
                Instructions::Read(_, _) => "Read",
                Instructions::ReadConst(_, _) => "ReadConst",
                Instructions::ReadPtr(_) => "Dereference",
                Instructions::ReserveStack(_, _) => "Reserve",
                Instructions::Return => "Return",
                Instructions::Sub(_, _, _) => "Subtract",
                Instructions::Write(_, _) => "Write",
                Instructions::WritePtr(_) => "WriteRef",
                Instructions::Cast(_, _) => "Casting",
                Instructions::Len(_) => "Length",
                Instructions::Type(_, _) => "TypeOf",
                Instructions::Jump(_) => "Jump",
                Instructions::Freeze => "Freeze",
                Instructions::Back => "Back",
                Instructions::Move(_, _) => "Move",
                Instructions::Sweep => "Sweep",
                Instructions::SweepUnoptimized => "SweepUnoptimized",
                Instructions::FillRange(_, _) => "ToRange",
                Instructions::CopyRange(_, _, _) => "CopyRange",
                Instructions::DynMethod(_, _, _) => "Method",
                Instructions::Break(_) => "Break",
                Instructions::Panic => "Panic",
                Instructions::Catch => "Catch",
                Instructions::CatchId(_) => "Catch",
                Instructions::DeleteCatch(_) => "DeleteCatch",
                Instructions::NonPrimitiveType(_, _) => "NonPrimitiveType",
                Instructions::StrNew => "StringNew",
                Instructions::Deallocate => "Deallocate",
                Instructions::IntoStr(_) => "IntoString",
                Instructions::DynReserve(_) => "ReserveDynamic",
                Instructions::DynArgument(_, _, _) => "ArgumentDynamic",
                Instructions::Neg(_) => "Negate",
                Instructions::WriteArg(_, _) => "WriteArgument",
                Instructions::ReadArg(_, _) => "ReadArgument",
                Instructions::OpenArgs => "OpenArguments",
                Instructions::CloseArgs => "CloseArguments",
                Instructions::NullCheck => "NullCheck",
            };
            write!(f, "{str}")
        }
    }
    /// holds information of where to jump after function call ends
    #[derive(Clone, Copy, Debug)]
    pub struct CallStack {
        pub reg_freeze: [Types; FREEZED_REG_SIZE],
        pub end: usize,
        pub code_ptr: usize,
        pub pointers_len: usize,
    }
}
pub mod runtime_error {
    use super::runtime_types::*;
    #[derive(Debug, Clone)]
    pub enum ErrTypes {
        /// Operation failed: Cross-type operation {var1:+}, {var2:+}
        CrossTypeOperation(Types, Types, Instructions),
        /// Operation failed: Wrong-type operation {var1:+}
        WrongTypeOperation(Types, Instructions),
        /// Invalid Type: {typ:#} must be of type '{operation:#}'
        InvalidType(Types, Types),
        /// Wrong type: Expected {exp:#}, found {found:#}
        Expected(Types, Types),
        /// Cast error: Can not implicitly cast type {type1:#} into type {type2:#}
        ImplicitCast(Types, Types),
        /// Stack overflow
        StackOverflow,
        /// Catch overflow
        CatchOwerflow,
        /// Method not found
        MethodNotFound,
        /// custom error message
        Message(String),
        /// Cannot use userdata
        CannotReadUserdata,
    }
    fn gen_message(header: String, line: Option<(usize, usize)>, err_no: u8) -> String {
        return if let Some(line) = line {
            //                    code                      header                      line     column
            format!("\x1b[90mErr{err_no:03}\x1b[0m \x1b[91m{header}\x1b[0m\n\x1b[90mAt: line {}, column {}.\x1b[0m", line.0, line.1)
        } else {
            format!("\x1b[90mErr{err_no:03}\x1b[0m \x1b[91m{header}\x1b[0m\n\x1b[90mLocation unspecified.\x1b[0m")
        };
    }
    pub fn get_message(kind: &ErrTypes, line: Option<(usize, usize)>) -> String {
        let data = match &kind {
            ErrTypes::CrossTypeOperation(var1, var2, instr) => (
                format!("Operation '{instr}' failed: Cross-type operation {var1:+}, {var2:+}"),
                0,
            ),
            ErrTypes::WrongTypeOperation(var1, instr) => (
                format!("Operation '{instr}' failed: Wrong-type operation {var1:+}"),
                1,
            ),
            ErrTypes::InvalidType(typ, operation) => (
                format!("Invalid Type: {typ:#} must be of type '{operation:#}'"),
                2,
            ),
            ErrTypes::Expected(exp, found) => {
                (format!("Wrong type: Expected {exp:#}, found {found:#}"), 3)
            }
            ErrTypes::ImplicitCast(type1, type2) => (
                format!("Cast error: Can not implicitly cast type {type1:#} into type {type2:#}"),
                4,
            ),
            ErrTypes::StackOverflow => (format!("Stack overflow"), 5), // TODO: impl
            ErrTypes::CatchOwerflow => (format!("Catch overflow"), 6),
            ErrTypes::MethodNotFound => (format!("Method not found"), 7),
            ErrTypes::Message(msg) => (msg.clone(), 8),
            ErrTypes::CannotReadUserdata => (format!("Cannot use userdata"), 9),
        };
        let message = gen_message(data.0, line, data.1);
        message
    }
}

pub mod user_data {
    /// Library defined data that lives inside the interpreter and can be accessed by any library
    ///
    /// To see, how to access this data, see doc string for fn as_any or fn as_any_mut
    pub trait UserData {
        /// return the label of the object
        /// this is used for debugging
        fn label(&self) -> &str;
        /// returns the id of the object
        /// this is useful for shared objects between libraries
        fn id(&self) -> usize;
        /// returns the id of the library
        /// this is used for debugging
        fn lib_id(&self) -> usize;
        /// describes how to aproach the object by the garbage collector
        fn gc_method(&self) -> &GcMethod;
        /// cleans up the object for garbage collection
        /// thanks to Rust this is not needed for most objects
        fn cleanup(&mut self);
        /// returns the object as any
        ///
        /// this method is used to downcast the object to its original type
        ///
        /// example:
        ///
        /// ```rust
        /// use runtime::user_data::*;
        ///
        /// struct MyStruct {
        ///    data: usize,
        /// }
        ///
        /// impl UserData for MyStruct {
        ///   // ...
        ///    fn as_any(&self) -> &dyn std::any::Any {
        ///       self
        ///   }
        /// }
        ///
        /// fn main() {
        ///   // runtime stores objects as Box<dyn UserData>
        ///   let my_struct: Box<dyn UserData> = Box::new(MyStruct { data: 0 });
        ///
        ///  // downcast to MyStruct
        ///  let my_struct: &MyStruct = my_struct.as_any().downcast_ref::<MyStruct>().unwrap();
        /// }
        /// ```
        fn as_any(&self) -> &dyn std::any::Any;
        /// returns the object as any mut
        ///
        /// this method is used to downcast the object to its original type
        ///
        /// example:
        ///
        /// ```rust
        /// use runtime::user_data::*;
        ///
        /// struct MyStruct {
        ///   data: usize,
        /// }
        ///
        /// impl UserData for MyStruct {
        ///  // ...
        ///  fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        ///    self
        ///  }
        /// }
        ///
        /// fn main() {
        ///  // runtime stores objects as Box<dyn UserData>
        ///  let mut my_struct: Box<dyn UserData> = Box::new(MyStruct { data: 0 });
        ///
        ///  // downcast to MyStruct
        ///  let my_struct: &mut MyStruct = my_struct.as_any_mut().downcast_mut::<MyStruct>().unwrap();
        /// }
        /// ```
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    }

    /// describes how to aproach the object by the garbage collector
    #[derive(Debug, Clone, Copy)]
    pub enum GcMethod {
        /// object is not garbage collected
        None,
        /// object can be safely deleted when it is no longer needed
        Gc,
        /// garbage collector runs cleanup function when it is no longer needed and then deletes the object
        Own,
    }

    /// null object
    pub struct Null;
    impl UserData for Null {
        fn id(&self) -> usize {
            0
        }
        fn lib_id(&self) -> usize {
            0
        }
        fn gc_method(&self) -> &GcMethod {
            &GcMethod::None
        }
        fn cleanup(&mut self) {}
        fn label(&self) -> &str {
            "EmptyUserData"
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }
}
