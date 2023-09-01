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

use std::io::Write;

use runtime::runtime_types::*;
use runtime::*;

pub struct Foo {
    file_handles: Vec<Option<std::fs::File>>,
    _id: usize,
}

impl lib::Library for Foo {
    fn call(
        &mut self,
        id: usize,
        mem: PublicData,
    ) -> Result<Types, runtime_error::ErrTypes> {
        let m = mem.memory;
        match id {
            // std::print
            0 => {
                if let Types::Pointer(u_size, PointerTypes::String) =
                    m.registers[runtime_types::POINTER_REG]
                {
                    let mut string = String::new();
                    for i in m.strings.pool[u_size].iter() {
                        string.push(*i);
                    }
                    print!("{}", string);
                    std::io::stdout().flush().unwrap();
                } else {
                    return Err(runtime_error::ErrTypes::Message(
                        "Invalid argument".to_owned(),
                    ));
                }
            }
            // std::println
            1 => {
                if let Types::Pointer(u_size, PointerTypes::String) =
                    m.registers[runtime_types::POINTER_REG]
                {
                    let mut string = String::new();
                    for i in m.strings.pool[u_size].iter() {
                        string.push(*i);
                    }
                    println!("{}", string);
                } else {
                    return Err(runtime_error::ErrTypes::Message(
                        "Invalid argument".to_owned(),
                    ));
                }
            }
            // std::read
            2 => {
                let mut input = String::new();
                match std::io::stdin().read_line(&mut input) {
                    Err(why) => {
                        return Err(runtime_error::ErrTypes::Message(format!(
                            "Couldn't read line: {}",
                            why
                        )))
                    }
                    Ok(_) => (),
                }
                m.strings.pool.push(input.chars().collect());
                return Ok(Types::Pointer(
                    m.strings.pool.len() - 1,
                    PointerTypes::String,
                ));
            }
            // std::args
            // returns array of strings
            3 => {
                // first get a vector of args
                let args: Vec<String> = std::env::args().collect();
                // allocate enough space for the array on the heap
                let obj = m.allocate_obj(args.len() + 1);
                // set the first element to the length of the array
                m.heap.data[obj][0] = Types::NonPrimitive(0);
                // iterate over the args
                for (i, arg) in args.iter().enumerate() {
                    // push the string to the string pool
                    let str = m.strings.from(arg.to_string().chars().collect());
                    // set the element in the array to the index of the string in the string pool
                    m.heap.data[obj][i + 1] = Types::Pointer(str, PointerTypes::String);
                }
                // return the pointer to the array
                return Ok(Types::Pointer(obj, PointerTypes::Object));
            }
            // std::runtimeArgs
            // returns array of strings passed to the runtime
            4 => {
                // first get a vector of args
                let args: Vec<String> = m.runtime_args.clone();
                // allocate enough space for the array on the heap
                let obj = m.allocate_obj(args.len() + 1);
                // set the first element to the length of the array
                m.heap.data[obj][0] = Types::NonPrimitive(0);
                // iterate over the args
                for (i, arg) in args.iter().enumerate() {
                    // push the string to the string pool
                    let str = m.strings.from(arg.to_string().chars().collect());
                    // set the element in the array to the index of the string in the string pool
                    m.heap.data[obj][i + 1] = Types::Pointer(str, PointerTypes::String);
                }
                // return the pointer to the array
                return Ok(Types::Pointer(obj, PointerTypes::Object));
            }
            _ => {
                unreachable!("Invalid function id")
            }
        }
        return Ok(runtime_types::Types::Void);
    }
}

#[no_mangle]
const name: &'static str = "io";


#[no_mangle]
fn register() -> String {
    return r#"
    
    fun print(msg=reg.ptr: string) > 0i
    fun println(msg=reg.ptr: string) > 1i
    fun args(): &[string] > 2i
    fun vmargs(): &[string] > 3i
    "#.to_string()
}

#[no_mangle]
pub fn init(_ctx: &mut Context, my_id: usize) -> Box<dyn lib::Library> {
    return Box::new(Foo {
        file_handles: Vec::new(),
        _id: my_id,
    });
}