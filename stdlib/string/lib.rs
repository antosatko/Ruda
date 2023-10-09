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
        match id {
            // string::concat
            0 => {
                // take two strings pointers from registers and concat them returning a new string pointer
                if let Types::Pointer(u_size, PointerTypes::String) = m.registers[POINTER_REG] {
                    if let Types::Pointer(u_size2, PointerTypes::String) = m.registers[GENERAL_REG1]
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
                // take a string pointer from registers and trim it returning a new string pointer
                if let Types::Pointer(u_size, PointerTypes::String) = m.registers[POINTER_REG] {
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
                // take a string pointer from registers and split it returning a new string pointer
                // also take a string pointer from GENERAL_REG1 and split it with it
                if let Types::Pointer(u_size, PointerTypes::String) = m.registers[POINTER_REG] {
                    if let Types::Pointer(u_size2, PointerTypes::String) = m.registers[GENERAL_REG1]
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
            // string::join
            3 => {
                // take a pointer to an array of strings and join them together
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
    "".to_string()
}

#[no_mangle]
pub fn init(_ctx: &mut Context, _my_id: usize) -> Box<dyn lib::Library> {
    return Box::new(String);
}
