extern crate runtime;

use runtime::runtime_types::*;
use runtime::*;

fn call(ctx: &mut Context, id: usize, lib_id: usize) -> Result<Types, runtime_error::ErrTypes> {
    let m = &mut ctx.memory;
    match id {
        0 => {
            if let Types::Pointer(u_size, PointerTypes::String) = m.registers[GENERAL_REG1] {
                let str = m.strings.to_string(u_size);
                let pos = match str == "" {
                    true => {
                        let pos = m.strings.from_str("5");
                        m.strings.pool[pos].clear();
                        pos
                    }
                    false => {
                        m.strings.from_str(&str)
                    }
                };
                return Ok(Types::Pointer(pos, PointerTypes::String))
            } else {
                return Err(runtime_error::ErrTypes::Message(format!(
                    "Invalid string pointer"
                )));
            }
        }
        // core::to_str 
        1 => {
            let value = m.registers[GENERAL_REG2];
            let str = value.to_str(m);
            let pos = m.strings.from_str(&str);
            return Ok(Types::Pointer(pos, PointerTypes::String));
        }
        // core::str_concat
        2 => {
            if let Types::Pointer(u_size, PointerTypes::String) = m.registers[GENERAL_REG1] {
                if let Types::Pointer(u_size1, PointerTypes::String) = m.registers[GENERAL_REG2] {
                    let str = m.strings.to_string(u_size) + &m.strings.to_str(u_size1);
                    let pos = m.strings.from_str(&str);
                    return Ok(Types::Pointer(pos, PointerTypes::String));
                } else {
                    return Err(runtime_error::ErrTypes::Message(format!(
                        "Invalid string pointer"
                    )));
                }
            } else {
                return Err(runtime_error::ErrTypes::Message(format!(
                    "Invalid string pointer"
                )));
            }
        }
        // core::str_cmp
        3 => {
            if let Types::Pointer(u_size, PointerTypes::String) = m.registers[GENERAL_REG1] {
                if let Types::Pointer(u_size1, PointerTypes::String) = m.registers[GENERAL_REG2] {
                    return Ok(Types::Bool(m.strings.to_str(u_size) == m.strings.to_str(u_size1)));
                } else {
                    return Err(runtime_error::ErrTypes::Message(format!(
                        "Invalid string pointer"
                    )));
                }
            } else {
                return Err(runtime_error::ErrTypes::Message(format!(
                    "Invalid string pointer"
                )));
            }
        }
        // core::arrlen
        5 => {
            let this = m.args()[0];
            if let Types::Pointer(u_size, PointerTypes::Object) = this {
                return Ok(Types::Uint(m.heap.data[u_size].len()));
            } else {
                return Err(runtime_error::ErrTypes::Message(format!(
                    "Invalid array pointer"
                )));
            }
        }
        // core::arrpush
        6 => {
            let this = m.args()[0];
            let arg = m.args()[1];
            if let Types::Pointer(u_size, PointerTypes::Object) = this {
                m.heap.data[u_size].push(arg.clone());
                return Ok(Types::Void);
            } else {
                return Err(runtime_error::ErrTypes::Message(format!(
                    "Invalid array pointer"
                )));
            }
        }

        7 => {
            let this = m.args()[0];
            let index = m.args()[1];
            if let Types::Pointer(u_size, PointerTypes::Object) = this {
                if let Types::Uint(idx) = index {
                    return Ok(m.heap.data[u_size].get(idx).cloned().unwrap_or(Types::Null));
                }
            }
            return Err(runtime_error::ErrTypes::Message("Invalid array access".to_string()));
        }
        8 => {
            let this = m.args()[0];
            let index = m.args()[1];
            let value = m.args()[2];
            if let Types::Pointer(u_size, PointerTypes::Object) = this {
                if let Types::Uint(idx) = index {
                    if idx < m.heap.data[u_size].len() {
                        m.heap.data[u_size][idx] = value.clone();
                        return Ok(Types::Void);
                    }
                }
            }
            return Err(runtime_error::ErrTypes::Message("Invalid array access".to_string()));
        }
        9 => {
            let this = m.args()[0];
            if let Types::Pointer(u_size, PointerTypes::Object) = this {
                return Ok(m.heap.data[u_size].pop().unwrap_or(Types::Null));
            }
            return Err(runtime_error::ErrTypes::Message("Invalid array pointer".to_string()));
        }
        10 => {
            let this = m.args()[0];
            let index = m.args()[1];
            if let Types::Pointer(u_size, PointerTypes::Object) = this {
                if let Types::Uint(idx) = index {
                    if idx < m.heap.data[u_size].len() {
                        return Ok(m.heap.data[u_size].remove(idx));
                    }else {
                        return Ok(Types::Null)
                    }
                }
            }
            return Err(runtime_error::ErrTypes::Message("Invalid array access".to_string()));
        }
        11 => {
            let this = m.args()[0];
            let i = m.args()[1];
            let j = m.args()[2];
            if let Types::Pointer(u_size, PointerTypes::Object) = this {
                if let (Types::Uint(idx1), Types::Uint(idx2)) = (i, j) {
                    let obj = &mut m.heap.data[u_size];
                    if idx1 >= obj.len() || idx2 >= obj.len() {
                        return Ok(Types::Void)
                    }
                    obj.swap(idx1, idx2);
                    return Ok(Types::Void);
                }
            }
            return Err(runtime_error::ErrTypes::Message("Invalid swap indices".to_string()));
        }
        id => unreachable!("Invalid function id: {id}"),
    }
}

#[no_mangle]
fn register() -> String {
    let mut result = r#"
    fun arrlen(self=reg.g1): uint > 5
    fun arrpush<T>(self=reg.g1, value=reg.g2: T) > 6
    fun arrget(self=reg.g1, index=reg.g2: uint): T? > 7
    fun arrset<T>(self=reg.g1, index=reg.g2: uint, value=reg.g3: T) > 8
    fun arrpop(self=reg.g1): T? > 9
    fun arrremove(self=reg.g1, index=reg.g2: uint): T? > 10
    fun arrswap(self=reg.g1, i=reg.g2: uint, j=reg.g3: uint) > 11
    "#.to_string();
    let primitives = ["int", "float", "bool", "null", "char", "uint"];
    /*for i in primitives.iter() {
        result.push_str(&format!("fun {}hello(self=reg.g1): {} > 4\n", i, i));
    }*/
    result
}

#[no_mangle]
pub fn init(_ctx: &mut Context, my_id: usize) -> fn(&mut Context, usize, usize) -> Result<Types, runtime_error::ErrTypes> {
    call
}
