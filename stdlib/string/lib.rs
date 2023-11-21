/**
 * Looks intimidating, ik
 *
 * to find the actual code, look for the match statement
 * or just ctrl+f for "std::<function>"
 *
 * there is no official documentation for writing Rusty danda libraries at the time of writing this
 * for more information, please refer to my github www.github.com/it-2001
 *
 */
extern crate runtime;

use runtime::runtime_types::*;
use runtime::*;

pub struct String;

impl lib::Library for String {
    fn call(
        &mut self,
        id: usize,
        mem: PublicData,
    ) -> Result<runtime_types::Types, runtime_error::ErrTypes> {
        let m = mem.memory;
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
            // string::concat
            0 => {
                let args = get_args!();
                // take two strings pointers from registers and concat them returning a new string pointer
                if let Types::Pointer(u_size, PointerTypes::String) = args[0] {
                    if let Types::Pointer(u_size2, PointerTypes::String) = args[1]
                    {
                        let mut new_string = m.strings.to_string(u_size);
                        new_string.push_str(m.strings.to_str(u_size2));
                        return Ok(Types::Pointer(
                            m.strings.from_str(&new_string),
                            PointerTypes::String,
                        ));
                    } else {
                        return Err(runtime_error::ErrTypes::Message(format!(
                            "Invalid second string pointer",
                        )));
                    }
                } else {
                    return Err(runtime_error::ErrTypes::Message(format!(
                        "Invalid first string pointer"
                    )));
                }
            }
            // string::trim
            1 => {
                let args = get_args!();
                // take a string pointer from registers and trim it returning a new string pointer
                if let Types::Pointer(u_size, PointerTypes::String) = args[0] {
                    let new_string = m.strings.to_str(u_size).trim().to_owned();
                    return Ok(Types::Pointer(
                        m.strings.from_str(&new_string),
                        PointerTypes::String,
                    ));
                } else {
                    return Err(runtime_error::ErrTypes::Message(format!(
                        "Invalid string pointer"
                    )));
                }
            }
            // string::split
            2 => {
                let args = get_args!();
                // take a string pointer from registers and split it returning a new string pointer
                // also take a string pointer from GENERAL_REG1 and split it with it
                if let Types::Pointer(u_size, PointerTypes::String) = args[0] {
                    if let Types::Pointer(u_size2, PointerTypes::String) = args[1]
                    {
                        let new_string = m.strings.to_string(u_size);
                        let split_string = m.strings.to_str(u_size2);
                        let split: Vec<&str> = new_string.split(&split_string).collect();
                        let obj = m.allocate_obj(split.len() + 1);
                        // set the first element to the length of the array
                        m.heap.data[obj][0] = Types::NonPrimitive(0);
                        for idx in m.strings.push_string_array(split) {
                            m.heap.data[obj][idx + 1] = Types::Pointer(idx, PointerTypes::String);
                        }
                        return Ok(Types::Pointer(obj, PointerTypes::Object));
                    } else {
                        return Err(runtime_error::ErrTypes::Message(format!(
                            "Invalid second string pointer",
                        )));
                    }
                } else {
                    return Err(runtime_error::ErrTypes::Message(format!(
                        "Invalid first string pointer"
                    )));
                }
            }
            // string::clone
            3 => {
                let args = get_args!();
                if let Types::Pointer(u_size, PointerTypes::String) = args[0] {
                    let str = m.strings.to_str(u_size).to_string();
                    let pos = m.strings.from_str(&str);
                    return Ok(Types::Pointer(pos, PointerTypes::String))
                } else {
                    return Err(runtime_error::ErrTypes::Message(format!(
                        "Invalid string pointer"
                    )));
                }
            }
            // string::len
            4 => {
                let args = get_args!();
                if let Types::Pointer(u_size, PointerTypes::String) = args[0] {
                    let len = m.strings.to_str(u_size).len();
                    return Ok(Types::Usize(len));
                } else {
                    return Err(runtime_error::ErrTypes::Message(format!(
                        "Invalid string pointer"
                    )));
                }
            }
            // string::parse
            5 => {
                let args = get_args!();
                if let Types::Pointer(u_size, PointerTypes::String) = args[0] {
                    let str = m.strings.to_str(u_size);
                    let num = str.parse::<f64>();
                    match num {
                        Ok(num) => {
                            return Ok(Types::Float(num));
                        }
                        Err(err) => {
                            return Err(runtime_error::ErrTypes::Message(format!(
                                "Couldn't parse string to float: {err}",
                            )));
                        }
                    }
                } else {
                    return Err(runtime_error::ErrTypes::Message(format!(
                        "Invalid string pointer"
                    )));
                }
            }

            _ => {
                unreachable!("Invalid function id")
            }
        }
        return Ok(runtime_types::Types::Void);
    }
}


#[no_mangle]
fn register() -> std::string::String {
    "
    fun concat(str=reg.ptr: string, other=reg.g1: string): string > 0i
    fun trim(str=reg.ptr: string): string > 1i
    fun split(str=reg.ptr: string, split=reg.g1: string): [string] > 2i
    fun clone(str=reg.ptr: string): string > 3i
    fun len(str=reg.ptr: string): usize > 4i
    fun parse(str=reg.ptr: string): float > 5i
    ".to_string()
}

#[no_mangle]
pub fn init(_ctx: &mut Context, _my_id: usize) -> Box<dyn lib::Library> {
    return Box::new(String);
}
