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

use std::fs::File;
use std::io::{Write, Read};

use runtime::runtime_types::*;
use runtime::user_data::UserData;
use runtime::*;

pub struct Foo {
    _id: usize,
}

impl lib::Library for Foo {
    fn call(&mut self, id: usize, mem: PublicData) -> Result<Types, runtime_error::ErrTypes> {
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
                if let Types::Pointer(u_size, PointerTypes::String) =
                    m.registers[runtime_types::POINTER_REG]
                {
                    let string = m.strings.to_string(u_size);
                    let file = match File::options().read(true).write(true).create(true).open(string) {
                        Err(why) => {
                            return Err(runtime_error::ErrTypes::Message(format!(
                                "Couldn't open file: {}",
                                why
                            )))
                        }
                        Ok(file) => file,
                    };
                    let idx = m.user_data.push(Box::new(FileH::new(file, self._id)));
                    return Ok(Types::Pointer(idx, PointerTypes::UserData));
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
                if let Types::Pointer(u_size, PointerTypes::UserData) =
                    m.registers[runtime_types::POINTER_REG]
                {
                    let any = &mut m.user_data.data[u_size];
                    let file = match FileH::from_ud(any.as_mut()) {
                        Ok(file) => file,
                        Err(why) => return Err(why),
                    };
                    file.cleanup();
                } else {
                    return Err(runtime_error::ErrTypes::Message(format!(
                        "Expected File handle, got {:#}",
                        m.registers[runtime_types::POINTER_REG]
                    )));
                }
            }
            // std::handle_read
            // takes index of file handle
            // returns string
            5 => {
                use std::io::prelude::*;
                if let Types::Pointer(u_size, PointerTypes::UserData) =
                    m.registers[runtime_types::POINTER_REG]
                {
                    let any = &mut m.user_data.data[u_size];
                    let file = match FileH::from_ud(any.as_mut()) {
                        Ok(file) => file,
                        Err(why) => return Err(why),
                    };
                    let mut contents = String::new();
                    match file.handle.read_to_string(&mut contents) {
                        Err(why) => {
                            return Err(runtime_error::ErrTypes::Message(format!(
                                "Couldn't read file: {}",
                                why
                            )))
                        }
                        Ok(_) => (),
                    }
                    return Ok(Types::Pointer(
                        m.strings.from_str(&contents),
                        PointerTypes::String,
                    ));
                } else {
                    return Err(runtime_error::ErrTypes::Message(format!(
                        "Expected file handle, got {:#}",
                        m.registers[runtime_types::POINTER_REG]
                    )));
                }
            }
            // std::handle_write
            // takes index of file handle
            // writes to file from register 1
            6 => {
                if let Types::Pointer(u_size, PointerTypes::UserData) =
                    m.registers[runtime_types::POINTER_REG]
                {
                    let any = &mut m.user_data.data[u_size];
                    let file = match FileH::from_ud(any.as_mut()) {
                        Ok(file) => file,
                        Err(why) => return Err(why),
                    };
                    if let Types::Pointer(u_size, PointerTypes::String) =
                        m.registers[runtime_types::GENERAL_REG1]
                    {
                        let string = m.strings.to_string(u_size);
                        match file.handle.write_all(string.as_bytes()) {
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
                        "Expected file handle, got {:#}",
                        m.registers[runtime_types::POINTER_REG]
                    )));
                }
            }
            // std::handle_append
            // takes index of file handle
            // appends to file from register 1
            7 => {
                use std::io::prelude::*;
                if let Types::Pointer(u_size, PointerTypes::UserData) =
                    m.registers[runtime_types::POINTER_REG]
                {
                    let any = &mut m.user_data.data[u_size];
                    let file = match FileH::from_ud(any.as_mut()) {
                        Ok(file) => file,
                        Err(why) => return Err(why),
                    };
                    if let Types::Pointer(u_size, PointerTypes::String) =
                        m.registers[runtime_types::GENERAL_REG1]
                    {
                        let string = m.strings.to_string(u_size);
                        match file.handle.write_all(string.as_bytes()) {
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
                        "Expected file handle, got {:#}",
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
    userdata File > 0i
    
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

    "#
    .to_string()
}

#[no_mangle]
pub fn init(_ctx: &mut Context, my_id: usize) -> Box<dyn lib::Library> {
    return Box::new(Foo { _id: my_id });
}

struct FileH {
    handle: std::fs::File,
    id: usize,
    lib_id: usize,
    gc_method: user_data::GcMethod,
    name: String,
}

use std::any::Any;
impl FileH {
    const ASSIGN_ID: usize = 0;

    fn new(file: std::fs::File, lib_id: usize) -> Self {
        Self {
            handle: file,
            lib_id,
            name: "File".to_owned(),
            id: Self::ASSIGN_ID + lib_id,
            gc_method: user_data::GcMethod::Gc,
        }
    }
    fn from_ud(ud: &mut dyn UserData) -> Result<&mut Self, runtime_error::ErrTypes> {
        return ud
            .as_any_mut()
            .downcast_mut::<Self>()
            .ok_or(runtime_error::ErrTypes::Message(
                "Invalid userdata type".to_owned(),
            ));
    }
}

impl UserData for FileH {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn id(&self) -> usize {
        self.id
    }

    fn lib_id(&self) -> usize {
        self.lib_id
    }

    fn gc_method(&self) -> &user_data::GcMethod {
        &self.gc_method
    }

    fn cleanup(&mut self) {
        
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
