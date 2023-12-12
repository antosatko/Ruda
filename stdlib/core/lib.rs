/**
 * Looks intimidating, ik
 *
 * to find the actual code, look for the match statement
 * or just ctrl+f for "std::print" or whatever you want to find
 *
 * there is no official documentation for writing Rusty danda libraries at the time of writing this
 * for more information, please refer to the main repository www.github.com/it-2001/Ruda
 *
 */
extern crate runtime;

use runtime::runtime_types::*;
use runtime::*;

fn call(ctx: &mut Context, id: usize, lib_id: usize) -> Result<Types, runtime_error::ErrTypes> {
        let m = &mut ctx.memory;
        macro_rules! get_args {
            () => {
                match m.args() {
                    Some(args) => args,
                    None => {
                        return Err(runtime_error::ErrTypes::Message(format!(
                            "Couldn't get args, this is probably a bug in the compiler",
                        )))
                    }
                }
            };
        }
        match id {
            // core::str_cpy
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
            // core::anyhello
            4 => {
                let args = get_args!();
                let n = args[0];
                println!("Hello, {:?}!", n);
            }
            _ => unreachable!("Invalid function id"),
        }
        return Ok(runtime_types::Types::Void);
    }

#[no_mangle]
fn register() -> String {
    let mut result = r#"
    "#.to_string();
    let primitives = ["int", "float", "bool", "null", "char", "uint"];
    for i in primitives.iter() {
        result.push_str(&format!("fun {}hello(self=reg.g1): {} > 4\n", i, i));
    }
    result
}

#[no_mangle]
pub fn init(_ctx: &mut Context, my_id: usize) -> fn(&mut Context, usize, usize) -> Result<Types, runtime_error::ErrTypes> {
    call
}