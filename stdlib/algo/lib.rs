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
            0 => {

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