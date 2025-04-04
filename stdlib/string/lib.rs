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

fn call(ctx: &mut Context, id: usize, lib_id: usize) -> Result<Types, runtime_error::ErrTypes> {
    let m = &mut ctx.memory;
    match id {
        // string::concat
        0 => {
            let args = m.args();
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
            let args = m.args();
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
            let args = m.args();
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
            let args = m.args();
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
            let args = m.args();
            if let Types::Pointer(u_size, PointerTypes::String) = args[0] {
                let len = m.strings.to_str(u_size).len();
                return Ok(Types::Uint(len));
            } else {
                return Err(runtime_error::ErrTypes::Message(format!(
                    "Invalid string pointer"
                )));
            }
        }
        // string::parse
        5 => {
            let args = m.args();
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
        // string::to_upper
        8 => {
            let args = m.args();
            if let Types::Pointer(u_size, PointerTypes::String) = args[0] {
                let original_str = m.strings.to_str(u_size);
                let upper_str = original_str.to_uppercase();
                return Ok(Types::Pointer(
                    m.strings.from_str(&upper_str),
                    PointerTypes::String,
                ));
            } else {
                return Err(runtime_error::ErrTypes::Message("Invalid string pointer".to_owned()));
            }
        }
        // string::to_lower
        9 => {
            let args = m.args();
            if let Types::Pointer(u_size, PointerTypes::String) = args[0] {
                let original_str = m.strings.to_str(u_size);
                let lower_str = original_str.to_lowercase();
                return Ok(Types::Pointer(
                    m.strings.from_str(&lower_str),
                    PointerTypes::String,
                ));
            } else {
                return Err(runtime_error::ErrTypes::Message("Invalid string pointer".to_owned()));
            }
        }
        // string::starts_with
        10 => {
            let args = m.args();
            if let Types::Pointer(u_size, PointerTypes::String) = args[0] {
                if let Types::Pointer(u_size2, PointerTypes::String) = args[1] {
                    let original_str = m.strings.to_str(u_size);
                    let prefix = m.strings.to_str(u_size2);
                    return Ok(Types::Bool(original_str.starts_with(prefix)));
                }
            }
            return Err(runtime_error::ErrTypes::Message("Invalid string pointers".to_owned()));
        }
        // string::ends_with
        11 => {
            let args = m.args();
            if let Types::Pointer(u_size, PointerTypes::String) = args[0] {
                if let Types::Pointer(u_size2, PointerTypes::String) = args[1] {
                    let original_str = m.strings.to_str(u_size);
                    let suffix = m.strings.to_str(u_size2);
                    return Ok(Types::Bool(original_str.ends_with(suffix)));
                }
            }
            return Err(runtime_error::ErrTypes::Message("Invalid string pointers".to_owned()));
        }
        _ => {
            unreachable!("Invalid function id")
        }
    }
    return Ok(runtime_types::Types::Void);
}

#[no_mangle]
fn register() -> std::string::String {
    "
    /// Concatenates two strings and returns a new string.
    /// The function takes two string pointers, concatenates the strings, and returns the result as a new string pointer.
    fun concat(str=reg.ptr: string, other=reg.g1: string): string > 0i

    /// Trims whitespace from both ends of a string.
    /// The function takes a string pointer, trims it, and returns the resulting string as a new pointer.
    fun trim(str=reg.ptr: string): string > 1i

    /// Splits a string into an array of strings based on a delimiter string.
    /// The function takes two string pointers, splits the first string using the second string as a delimiter, and returns the result as an array of strings.
    fun split(str=reg.ptr: string, split=reg.g1: string): [string] > 2i

    /// Creates a clone of a string.
    /// The function takes a string pointer and returns a new pointer with a copy of the original string.
    fun clone(str=reg.ptr: string): string > 3i

    /// Returns the length of the string.
    /// The function takes a string pointer and returns the length of the string as an unsigned integer.
    fun len(str=reg.ptr: string): usize > 4i

    /// Parses a string to a floating point number.
    /// The function takes a string pointer and attempts to parse it to a float, returning the result as a floating point number.
    fun parse(str=reg.ptr: string): float > 5i


    /// Converts a string to uppercase.
    /// The function takes a string pointer and returns the string converted to uppercase.
    fun to_upper(str=reg.ptr: string): string > 8i

    /// Converts a string to lowercase.
    /// The function takes a string pointer and returns the string converted to lowercase.
    fun to_lower(str=reg.ptr: string): string > 9i

    /// Checks if the string starts with a given prefix.
    /// The function takes two string pointers, the first is the original string,
    /// and the second is the prefix to check.
    /// It returns a boolean value indicating whether the string starts with the prefix.
    fun starts_with(str=reg.ptr: string, prefix=reg.g1: string): bool > 10i

    /// Checks if the string ends with a given suffix.
    /// The function takes two string pointers, the first is the original string,
    /// and the second is the suffix to check.
    /// It returns a boolean value indicating whether the string ends with the suffix.
    fun ends_with(str=reg.ptr: string, suffix=reg.g1: string): bool > 11i
    ".to_string()
}

#[no_mangle]
pub fn init(_ctx: &mut Context, my_id: usize) -> fn(&mut Context, usize, usize) -> Result<Types, runtime_error::ErrTypes> {
    call
}