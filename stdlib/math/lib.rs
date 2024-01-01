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
                let args = m.args();
                if let Types::Float(x) = args[0] {
                    return Ok(Types::Float(x.sin()));
                }
                return Err(runtime_error::ErrTypes::InvalidType(args[0], Types::Float(0.)));
            }
            1 => {
                let args = m.args();
                if let Types::Float(x) = args[0] {
                    return Ok(Types::Float(x.cos()));
                }
                return Err(runtime_error::ErrTypes::InvalidType(args[0], Types::Float(0.)));
            }
            2 => {
                let args = m.args();
                if let Types::Float(y) = args[0] {
                    if let Types::Float(x) = args[1] {
                        return Ok(Types::Float(y.atan2(x)));
                    }
                    return Err(runtime_error::ErrTypes::InvalidType(args[1], Types::Float(0.)));
                }
                return Err(runtime_error::ErrTypes::InvalidType(args[0], Types::Float(0.)));
            }
            3 => {
                let args = m.args();
                if let Types::Float(x) = args[0] {
                    return Ok(Types::Float(x.atan()));
                }
                return Err(runtime_error::ErrTypes::InvalidType(args[0], Types::Float(0.)));
            }
            4 => {
                let args = m.args();
                if let Types::Float(x) = args[0] {
                    return Ok(Types::Float(x.sqrt()));
                }
                return Err(runtime_error::ErrTypes::InvalidType(args[0], Types::Float(0.)));
            }
            _ => unreachable!("Invalid function id"),
        }
        return Ok(runtime_types::Types::Void);
    }


#[no_mangle]
fn register() -> String {
    r#"
    fun sin(x=reg.g1: float): float > 0i
    fun cos(x=reg.g1: float): float > 1i
    fun atan2(y=reg.g1: float, x=reg.g2: float): float > 2i
    fun atan(x=reg.g1: float): float > 3i
    fun sqrt(x=reg.g1: float): float > 4i
    "#.to_string()
}

#[no_mangle]
pub fn init(_ctx: &mut Context, my_id: usize) -> fn(&mut Context, usize, usize) -> Result<Types, runtime_error::ErrTypes> {
    call
}