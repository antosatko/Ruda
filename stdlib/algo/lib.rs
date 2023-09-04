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


const STRUCT_ID: usize = 0;
pub struct Algo{
    my_id: usize,
}

impl lib::Library for Algo {
    fn call(
        &mut self,
        id: usize,
        mem: PublicData,
    ) -> Result<Types, runtime_error::ErrTypes> {
        let m = mem.memory;
        match id {
            // Array::constructor
            0 => {
                // allocate memory for the object
                let raw_ptr = m.allocate_obj(1);
                // create a pointer to the object
                let ptr = Types::Pointer(raw_ptr, PointerTypes::Object);
                // set the type id of the object
                let type_id = Types::NonPrimitive(self.my_id + STRUCT_ID);
                m.heap.data[raw_ptr][0] = type_id;
                // return the pointer
                return Ok(ptr)
            }
            // Array::push
            1 => {
                if let Types::Pointer(loc, kind) = m.registers[POINTER_REG] {
                    if !kind.is_object() || !m.verify_obj(loc, self.my_id + STRUCT_ID) {
                        return Err(runtime_error::ErrTypes::Message(
                            "Expected object pointer".to_string(),
                        ));
                    }
                    m.resize_obj_relative(loc, 1);
                    let last = m.obj_len(loc);
                    m.heap.data[loc][last] = m.registers[GENERAL_REG1];
                }
            }
            // Array::pop
            2 => {
                if let Types::Pointer(loc, kind) = m.registers[POINTER_REG] {
                    if !kind.is_object() || !m.verify_obj(loc, self.my_id + STRUCT_ID) {
                        return Err(runtime_error::ErrTypes::Message(
                            "Expected object pointer".to_string(),
                        ));
                    }
                    let last = m.obj_len(loc);
                    if last == 0 {
                        return Err(runtime_error::ErrTypes::Message(
                            "Array is empty".to_string(),
                        ));
                    }
                    let value = m.heap.data[loc][last - 1];
                    m.registers[GENERAL_REG1] = m.heap.data[loc][last - 1];
                    m.resize_obj_relative(loc, -1);
                    // return the popped value
                    return Ok(value);
                }
            }
            // Array::remove
            3 => {
                // kinda complicated so maybe next time?
            }
            // Array::len
            4 => {
                if let Types::Pointer(loc, kind) = m.registers[POINTER_REG] {
                    if !kind.is_object() || !m.verify_obj(loc, self.my_id + STRUCT_ID) {
                        return Err(runtime_error::ErrTypes::Message(
                            "Expected object pointer".to_string(),
                        ));
                    }
                    m.registers[GENERAL_REG1] = Types::Int(m.obj_len(loc) as i64);
                }
            }
            _ => unreachable!("Invalid function id"),
        }
        return Ok(runtime_types::Types::Void);
    }
}

#[no_mangle]
fn register() -> String {
r#"
struct Array<T> > 0i {
    data: &T,
}

impl Array {
    fun constructor() > 0i
    fun push(&self=reg.ptr, val=reg.g1: T) > 1i
    fun pop(&self=reg.ptr)!: T > 2i
    fun remove(&self=reg.ptr, index=reg.g1: int)!: T > 3i
    fun len(&self=reg.ptr): int > 4i


    // overload [ (index=reg.g1: int)!: &T > 28i // fix overloads first
}

impl Array trait Iterator {
    fun next(&self=reg.ptr): T? > 29i
}

trait Iterator<T> > 1i {
    fun next(&self=reg.ptr): T?
}
"#.to_string()
}


/// my_id is the id of the library
/// this is used to identify the types produced by the library
/// for example, if the library produces a type with id 0, then the type will be 0 + my_id
/// ids must be given to structs, enums, and traits
#[no_mangle]
pub fn init(_ctx: &mut Context, my_id: usize) -> Box<dyn lib::Library> {
    return Box::new(Algo {my_id});
}