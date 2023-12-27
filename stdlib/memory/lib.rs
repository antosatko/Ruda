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

fn call(ctx: &mut Context, id: usize, lib_id: usize) -> Result<Types, runtime_error::ErrTypes> {
        let m = &mut ctx.memory;
        match id {
            0 => {
                let status = m.gc.disabled;
                return Ok(Types::Bool(status));
            }
            1 => {
                let args = m.args();
                let status = if let Types::Bool(status) = args[0] {
                    status
                } else {
                    return Err(runtime_error::ErrTypes::InvalidType(args[0], Types::Bool(false)));
                };
                m.gc.disabled = status;
            }
            2 => {
                let freed_before = m.gc.memory_swept;
                m.gc_sweep_unoptimized();
                let freed_after = m.gc.memory_swept;
                let freed = freed_after - freed_before;
                return Ok(Types::Uint(freed));
            }
            3 => {
                todo!()
            }
            4 => {
                let freed = m.gc.memory_swept;
                return Ok(Types::Uint(freed));
            }
            _ => unreachable!("Invalid function id"),
        }
        return Ok(runtime_types::Types::Void);
    }


#[no_mangle]
fn register() -> String {
r#"
userdata Gc > 0i {
    /// returns the current status of the garbage collector (on/off)
    fun disabled(): bool > 0i
    /// sets the status of the garbage collector (on/off)
    fun set(status=reg.G1: bool) > 1i
    /// sweeps the memory and returns the amount of memory freed
    fun sweep(): uint > 2i

    // -- control functions --
    fun limitDuration(duration=reg.G1: uint) > 3i

    // -- statistics --
    /// returns the total amount of memory freed
    fun freed(): uint > 4i
    
}

"#.to_string()
}

#[no_mangle]
pub fn init(_ctx: &mut Context, my_id: usize) -> fn(&mut Context, usize, usize) -> Result<Types, runtime_error::ErrTypes> {
    call
}