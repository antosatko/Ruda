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


/// returns (this, array_ptr, data_loc, len)
fn read_array(m: &mut Memory) -> (Types, Types, usize, usize) {
    let this = m.registers[POINTER_REG];
    let array_ptr = m.index(this, 1);
    let data_loc = array_ptr.ptr_loc();
    let len = m.obj_len(data_loc);
    return (this, array_ptr, data_loc, len)
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
                let this = m.registers[POINTER_REG];
                let raw_ptr = m.allocate_obj(1);
                let ptr = Types::Pointer(raw_ptr, PointerTypes::Object);
                let type_id = Types::NonPrimitive(self.my_id + STRUCT_ID);
                m.heap.data[raw_ptr][0] = type_id;
                let _ = m.write_idx(this.ptr_loc(), &mut this.kind(), 1, &ptr);
            }
            // Array::push
            1 => {
                let (_, _, loc, len) = read_array(m);
                m.grow_obj(loc, 1);
                m.heap.data[loc][len] = m.registers[GENERAL_REG1];
            }
            // Array::pop
            2 => {
                let (_, _, loc, len) = read_array(m);
                if len < 2 {
                    return Ok(Types::Null)
                }
                let result = m.heap.data[loc][len - 1];
                m.grow_obj(loc, -1);
                return Ok(result);
            }
            // Array::remove
            3 => {
                // kinda complicated so maybe next time?
            }
            // Array::len
            4 => {
                let (_, _, _, len) = read_array(m);
                return Ok(Types::Usize(len - 1))
            }
            _ => unreachable!("Invalid function id"),
        }
        return Ok(runtime_types::Types::Void);
    }
}

#[no_mangle]
fn register() -> String {
r#"
struct Array<T(Primitive)> > 0i {
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