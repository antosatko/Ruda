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
            // std::file_read
            0 => {
                use std::fs::File;
                use std::io::prelude::*;
                if let Types::Pointer(u_size, PointerTypes::String) =
                    m.registers[runtime_types::POINTER_REG]
                {
                    let string = m.strings.to_string(u_size);
                    let mut file = match File::open(string) {
                        Err(why) => {
                            return Err(runtime_error::ErrTypes::Message(format!(
                                "Couldn't open file: {}",
                                why
                            )))
                        }
                        Ok(file) => file,
                    };
                    let mut contents = String::new();
                    match file.read_to_string(&mut contents) {
                        Err(why) => {
                            return Err(runtime_error::ErrTypes::Message(format!(
                                "Couldn't read file: {}",
                                why
                            )))
                        }
                        Ok(_) => (),
                    }
                    m.strings.pool.push(contents.chars().collect());
                    return Ok(Types::Pointer(
                        m.strings.pool.len() - 1,
                        PointerTypes::String,
                    ));
                } else {
                    return Err(runtime_error::ErrTypes::Message(
                        "Invalid argument".to_owned(),
                    ));
                }
            }
            // std::file_write
            1 => {
                use std::fs::File;
                use std::io::prelude::*;
                if let Types::Pointer(u_size, PointerTypes::String) =
                    m.registers[runtime_types::POINTER_REG]
                {
                    let string = m.strings.to_string(u_size);
                    let mut file = match File::create(string) {
                        Err(why) => {
                            return Err(runtime_error::ErrTypes::Message(format!(
                                "Couldn't create file: {}",
                                why
                            )))
                        }
                        Ok(file) => file,
                    };
                    if let Types::Pointer(u_size, PointerTypes::String) =
                        m.registers[runtime_types::GENERAL_REG1]
                    {
                        let string = m.strings.to_string(u_size);
                        match file.write_all(string.as_bytes()) {
                            Err(why) => {
                                return Err(runtime_error::ErrTypes::Message(format!(
                                    "Couldn't write to file: {}",
                                    why
                                )))
                            }
                            Ok(_) => (),
                        }
                    } else {
                        return Err(runtime_error::ErrTypes::Message(
                            "Invalid argument".to_owned(),
                        ));
                    }
                } else {
                    return Err(runtime_error::ErrTypes::Message(
                        "Invalid argument".to_owned(),
                    ));
                }
            }
            // std::file_append
            2 => {
                use std::fs::OpenOptions;
                use std::io::prelude::*;
                if let Types::Pointer(u_size, PointerTypes::String) =
                    m.registers[runtime_types::POINTER_REG]
                {
                    let string = m.strings.to_string(u_size);
                    let mut file = match OpenOptions::new().append(true).open(string) {
                        Err(why) => {
                            return Err(runtime_error::ErrTypes::Message(format!(
                                "Couldn't open file: {}",
                                why
                            )))
                        }
                        Ok(file) => file,
                    };
                    if let Types::Pointer(u_size, PointerTypes::String) =
                        m.registers[runtime_types::GENERAL_REG1]
                    {
                        let string = m.strings.to_string(u_size);
                        match file.write_all(string.as_bytes()) {
                            Err(why) => {
                                return Err(runtime_error::ErrTypes::Message(format!(
                                    "Couldn't write to file: {}",
                                    why
                                )))
                            }
                            Ok(_) => (),
                        }
                    } else {
                        return Err(runtime_error::ErrTypes::Message(format!(
                            "String pointer expected, got {:#}",
                            m.registers[runtime_types::GENERAL_REG1]
                        )));
                    }
                } else {
                    return Err(runtime_error::ErrTypes::Message(
                        "Invalid argument".to_owned(),
                    ));
                }
            }
            // std::file_open
            // returns index of file handle
            3 => {
                use std::fs::File;
                if let Types::Pointer(u_size, PointerTypes::String) =
                    m.registers[runtime_types::POINTER_REG]
                {
                    let string = m.strings.to_string(u_size);
                    let file = match File::open(string) {
                        Err(why) => {
                            return Err(runtime_error::ErrTypes::Message(format!(
                                "Couldn't open file: {}",
                                why
                            )))
                        }
                        Ok(file) => file,
                    };
                    self.file_handles.push(Some(file));
                    return Ok(Types::Usize(self.file_handles.len() - 1));
                } else {
                    return Err(runtime_error::ErrTypes::Message(
                        "Invalid argument".to_owned(),
                    ));
                }
            }
            // std::file_close
            // takes index of file handle
            // returns bool
            4 => {
                if let Types::Usize(u_size) = m.registers[runtime_types::POINTER_REG] {
                    if u_size >= self.file_handles.len() {
                        return Err(runtime_error::ErrTypes::Message(
                            "Invalid file handle".to_owned(),
                        ));
                    }
                    if self.file_handles[u_size].is_none() {
                        return Err(runtime_error::ErrTypes::Message(
                            "File handle already closed".to_owned(),
                        ));
                    }
                    self.file_handles[u_size] = None;
                } else {
                    return Err(runtime_error::ErrTypes::Message(format!(
                        "File handle must be usize, got {:#}",
                        m.registers[runtime_types::POINTER_REG]
                    )));
                }
            }
            // std::handle_read
            // takes index of file handle
            // returns string
            5 => {
                use std::io::prelude::*;
                if let Types::Usize(u_size) = m.registers[runtime_types::POINTER_REG] {
                    if u_size >= self.file_handles.len() {
                        return Err(runtime_error::ErrTypes::Message(
                            "Invalid file handle".to_owned(),
                        ));
                    }
                    if self.file_handles[u_size].is_none() {
                        return Err(runtime_error::ErrTypes::Message(
                            "File handle already closed".to_owned(),
                        ));
                    }
                    let mut file = self.file_handles[u_size].as_ref().unwrap();
                    let mut contents = String::new();
                    match file.read_to_string(&mut contents) {
                        Err(why) => {
                            return Err(runtime_error::ErrTypes::Message(format!(
                                "Couldn't read file: {}",
                                why
                            )))
                        }
                        Ok(_) => (),
                    }
                    m.strings.pool.push(contents.chars().collect());
                    return Ok(Types::Pointer(
                        m.strings.pool.len() - 1,
                        PointerTypes::String,
                    ));
                } else {
                    return Err(runtime_error::ErrTypes::Message(format!(
                        "File handle must be usize, got {:#}",
                        m.registers[runtime_types::POINTER_REG]
                    )));
                }
            }
            // std::handle_write
            // takes index of file handle
            // writes to file from register 1
            6 => {
                use std::io::prelude::*;
                if let Types::Usize(u_size) = m.registers[runtime_types::POINTER_REG] {
                    if u_size >= self.file_handles.len() {
                        return Err(runtime_error::ErrTypes::Message(
                            "Invalid file handle".to_owned(),
                        ));
                    }
                    if self.file_handles[u_size].is_none() {
                        return Err(runtime_error::ErrTypes::Message(
                            "File handle already closed".to_owned(),
                        ));
                    }
                    let mut file = self.file_handles[u_size].as_ref().unwrap();
                    if let Types::Pointer(u_size, PointerTypes::String) =
                        m.registers[runtime_types::GENERAL_REG1]
                    {
                        let string = m.strings.to_string(u_size);
                        match file.write_all(string.as_bytes()) {
                            Err(why) => {
                                return Err(runtime_error::ErrTypes::Message(format!(
                                    "Couldn't write to file: {}",
                                    why
                                )))
                            }
                            Ok(_) => (),
                        }
                    } else {
                        return Err(runtime_error::ErrTypes::Message(format!(
                            "String pointer expected, got {:#}",
                            m.registers[runtime_types::GENERAL_REG1]
                        )));
                    }
                } else {
                    return Err(runtime_error::ErrTypes::Message(format!(
                        "File handle must be usize, got {:#}",
                        m.registers[runtime_types::POINTER_REG]
                    )));
                }
            }
            // std::handle_append
            // takes index of file handle
            // appends to file from register 1
            7 => {
                use std::io::prelude::*;
                if let Types::Usize(u_size) = m.registers[runtime_types::POINTER_REG] {
                    if u_size >= self.file_handles.len() {
                        return Err(runtime_error::ErrTypes::Message(
                            "Invalid file handle".to_owned(),
                        ));
                    }
                    if self.file_handles[u_size].is_none() {
                        return Err(runtime_error::ErrTypes::Message(
                            "File handle already closed".to_owned(),
                        ));
                    }
                    let mut file = self.file_handles[u_size].as_ref().unwrap();
                    if let Types::Pointer(u_size, PointerTypes::String) =
                        m.registers[runtime_types::GENERAL_REG1]
                    {
                        let string = m.strings.to_string(u_size);
                        match file.write_all(string.as_bytes()) {
                            Err(why) => {
                                return Err(runtime_error::ErrTypes::Message(format!(
                                    "Couldn't write to file: {}",
                                    why
                                )))
                            }
                            Ok(_) => (),
                        }
                    } else {
                        return Err(runtime_error::ErrTypes::Message(format!(
                            "String pointer expected, got {:#}",
                            m.registers[runtime_types::GENERAL_REG1]
                        )));
                    }
                } else {
                    return Err(runtime_error::ErrTypes::Message(format!(
                        "File handle must be usize, got {:#}",
                        m.registers[runtime_types::POINTER_REG]
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
fn register() -> String {
    
    
    
    
    
    
    r#"
    type File = usize > 0i
    
    impl File {
        fun read(&self=reg.ptr): string > 5i
        fun write(&self=reg.ptr, data=reg.G1:string)! > 6i
        fun append(&self=reg.ptr, data=reg.G1:string)! > 7i
        fun close(&self=reg.ptr)! > 4i
    }
    
    fun fileRead(fileName=reg.ptr: string): string > 0i
    fun fileWrite(fileName=reg.ptr: string, data=reg.G1: string)! > 1i
    fun fileAppend(fileName=reg.ptr: string, data=reg.G1: string)! > 2i
    fun fileOpen(fileName=reg.ptr: string)!: File > 3i

    "#.to_string()
}

#[no_mangle]
pub fn init(_ctx: &mut Context, my_id: usize) -> Box<dyn lib::Library> {
    return Box::new(Foo {
        file_handles: Vec::new(),
        _id: my_id,
    });
}