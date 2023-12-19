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

use console::Key;
use runtime::runtime_types::*;
use runtime::*;

fn call(ctx: &mut Context, id: usize, lib_id: usize) -> Result<Types, runtime_error::ErrTypes> {
        let m = &mut ctx.memory;
        match id {
            // std::print
            0 => {
                let args = m.args();
                if let Types::Pointer(u_size, PointerTypes::String) =
                    args[0]
                {
                    let string = &m.strings.pool[u_size];
                    print!("{}", string);
                    std::io::stdout().flush().unwrap();
                } else {
                    return Err(runtime_error::ErrTypes::Message(
                        "Invalid argument".to_owned(),
                    ));
                }
                Ok(Types::Void)
            }
            // std::println
            1 => {
                let args = m.args();
                if let Types::Pointer(u_size, PointerTypes::String) =
                    args[0]
                {
                    let string = &m.strings.pool[u_size];
                    println!("{}", string);
                } else {
                    return Err(runtime_error::ErrTypes::Message(
                        format!("Invalid argument {:?}", m.registers[runtime_types::POINTER_REG]),
                    ));
                }
                Ok(Types::Void)
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
                    let str = m.strings.from_str(&arg);
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
                    let str = m.strings.from_str(&arg);
                    // set the element in the array to the index of the string in the string pool
                    m.heap.data[obj][i + 1] = Types::Pointer(str, PointerTypes::String);
                }
                // return the pointer to the array
                return Ok(Types::Pointer(obj, PointerTypes::Object));
            }
            // std::readln
            5 => {
                let mut input = String::new();
                match std::io::stdin().read_line(&mut input) {
                    Err(why) => {
                        return Err(runtime_error::ErrTypes::Message(format!(
                            "Couldn't read input: {}",
                            why
                        )))
                    }
                    Ok(_) => (),
                }
                m.strings.from_string(input.trim_end().to_owned());
                return Ok(Types::Pointer(
                    m.strings.pool.len() - 1,
                    PointerTypes::String,
                ));
            }
            // std::getChar
            6 => {
                use console::Term;
                let term = Term::stdout();
                // wait for a keypress
                let key = loop {match term.read_key() {
                    Ok(Key::Char(c)) => break c,
                    Ok(Key::Enter) => break '\n',
                    Ok(_) => continue,
                    Err(why) => {
                        return Err(runtime_error::ErrTypes::Message(format!(
                            "Couldn't read key: {}",
                            why
                        )))
                    }
                }};
                // return the key
                return Ok(Types::Char(key));
            }
            // std::clear
            7 => {
                use console::Term;
                let term = Term::stdout();
                // clear the terminal
                match term.clear_screen() {
                    Ok(_) => (),
                    Err(why) => {
                        return Err(runtime_error::ErrTypes::Message(format!(
                            "Couldn't clear screen: {}",
                            why
                        )))
                    }
                
                };
                return Ok(Types::Void);
            }
            _ => {
                unreachable!("Invalid function id")
            }
        }
    }




#[no_mangle]
fn register() -> String {
    return r#"
    

    
    fun print(msg=reg.ptr: string) > 0i
    fun println(msg=reg.ptr: string) > 1i
    fun input(): string > 2i
    fun args(): [string] > 3i
    fun vmargs(): [string] > 4i
    fun inputln(): string > 5i
    fun getChar(): char > 6i
    fun clear()! > 7i
    "#.to_string()
}

#[no_mangle]
pub fn init(_ctx: &mut Context, my_id: usize) -> fn(&mut Context, usize, usize) -> Result<Types, runtime_error::ErrTypes> {
    call
}