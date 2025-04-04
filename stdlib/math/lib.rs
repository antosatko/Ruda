/*
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
use std::f64::consts;

fn call(ctx: &mut Context, id: usize, lib_id: usize) -> Result<Types, runtime_error::ErrTypes> {
    let m = &mut ctx.memory;
    match id {
        0 => {
            let args = m.args();
            if let Types::Float(x) = args[0] {
                return Ok(Types::Float(x.sin()));
            }
            Err(runtime_error::ErrTypes::InvalidType(args[0], Types::Float(0.)))
        }
        1 => {
            let args = m.args();
            if let Types::Float(x) = args[0] {
                return Ok(Types::Float(x.cos()));
            }
            Err(runtime_error::ErrTypes::InvalidType(args[0], Types::Float(0.)))
        }
        2 => {
            let args = m.args();
            if let Types::Float(y) = args[0] {
                if let Types::Float(x) = args[1] {
                    return Ok(Types::Float(y.atan2(x)));
                }
                Err(runtime_error::ErrTypes::InvalidType(args[1], Types::Float(0.)))?
            }
            Err(runtime_error::ErrTypes::InvalidType(args[0], Types::Float(0.)))
        }
        3 => {
            let args = m.args();
            if let Types::Float(x) = args[0] {
                return Ok(Types::Float(x.atan()));
            }
            Err(runtime_error::ErrTypes::InvalidType(args[0], Types::Float(0.)))
        }
        4 => {
            let args = m.args();
            if let Types::Float(x) = args[0] {
                return Ok(Types::Float(x.sqrt()));
            }
            Err(runtime_error::ErrTypes::InvalidType(args[0], Types::Float(0.)))
        }
        5 => {
            let args = m.args();
            if let Types::Float(x) = args[0] {
                return Ok(Types::Float(x.tan()));
            }
            Err(runtime_error::ErrTypes::InvalidType(args[0], Types::Float(0.)))
        }
        6 => {
            let args = m.args();
            if let Types::Float(x) = args[0] {
                return Ok(Types::Float(x.exp()));
            }
            Err(runtime_error::ErrTypes::InvalidType(args[0], Types::Float(0.)))
        }
        7 => {
            let args = m.args();
            if let Types::Float(x) = args[0] {
                return Ok(Types::Float(x.abs()));
            }
            Err(runtime_error::ErrTypes::InvalidType(args[0], Types::Float(0.)))
        }
        8 => {
            let args = m.args();
            if let Types::Float(x) = args[0] {
                if let Types::Float(y) = args[1] {
                    return Ok(Types::Float(x.powf(y)));
                }
                Err(runtime_error::ErrTypes::InvalidType(args[1], Types::Float(0.)))?
            }
            Err(runtime_error::ErrTypes::InvalidType(args[0], Types::Float(0.)))
        }
        9 => return Ok(Types::Float(consts::PI)),
        10 => return Ok(Types::Float(consts::E)),
        11 => return Ok(Types::Float(consts::TAU)),
        12 => {
            let args = m.args();
            if let Types::Float(x) = args[0] {
                return Ok(Types::Float(x.round()));
            }
            Err(runtime_error::ErrTypes::InvalidType(args[0], Types::Float(0.)))
        }
        13 => {
            let args = m.args();
            if let Types::Float(x) = args[0] {
                return Ok(Types::Float(x.floor()));
            }
            Err(runtime_error::ErrTypes::InvalidType(args[0], Types::Float(0.)))
        }
        14 => {
            let args = m.args();
            if let Types::Float(x) = args[0] {
                return Ok(Types::Float(x.ceil()));
            }
            Err(runtime_error::ErrTypes::InvalidType(args[0], Types::Float(0.)))
        }
        _ => unreachable!("Invalid function id"),
    }
}

#[no_mangle]
fn register() -> String {
    r#"
    /// Sine of x (sin(x))
    fun sin(x=reg.g1: float): float > 0i
    /// Cosine of x (cos(x))
    fun cos(x=reg.g1: float): float > 1i
    /// Arctangent of y/x (atan2(y, x))
    fun atan2(y=reg.g1: float, x=reg.g2: float): float > 2i
    /// Arctangent of x (atan(x))
    fun atan(x=reg.g1: float): float > 3i
    /// Square root of x (sqrt(x))
    fun sqrt(x=reg.g1: float): float > 4i
    /// Tangent of x (tan(x))
    fun tan(x=reg.g1: float): float > 5i
    /// Exponential of x (exp(x))
    fun exp(x=reg.g1: float): float > 6i
    /// Absolute value of x (abs(x))
    fun abs(x=reg.g1: float): float > 7i
    /// x raised to the power of y (pow(x, y))
    fun pow(x=reg.g1: float, y=reg.g2: float): float > 8i
    /// The constant π (pi)
    fun pi(): float > 9i
    /// The constant e
    fun e(): float > 10i
    /// The constant τ (tau)
    fun tau(): float > 11i
    /// Rounds x to the nearest integer (round(x))
    fun round(x=reg.g1: float): float > 12i
    /// Rounds x down to the nearest integer (floor(x))
    fun floor(x=reg.g1: float): float > 13i
    /// Rounds x up to the nearest integer (ceil(x))
    fun ceil(x=reg.g1: float): float > 14i
    /// Converts degrees to radians (to_radians(x))
    fun to_radians(x=reg.g1: float): float > 15i
    /// Converts radians to degrees (to_degrees(x))
    fun to_degrees(x=reg.g1: float): float > 16i
    "#.to_string()
}

#[no_mangle]
pub fn init(_ctx: &mut Context, my_id: usize) -> fn(&mut Context, usize, usize) -> Result<Types, runtime_error::ErrTypes> {
    call
}
