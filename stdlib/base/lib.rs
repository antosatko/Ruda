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

pub struct DynLib {
}

impl lib::Library for DynLib {
    fn call(
        &mut self,
        id: usize,
        mem: PublicData,
    ) -> Result<Types, runtime_error::ErrTypes> {
        let _m = mem.memory;
        macro_rules! get_args {
            () => {
                match m.args() {
                    Some(args) => args,
                    None => {
                        return Err(runtime_error::ErrTypes::Message(format!(
                            "Couldn't get args, this is probably a bug in the compiler",
                        )))
                    }
                }
            };
        }
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

    "#.to_string()
}

#[no_mangle]
pub fn init(_ctx: &mut Context, _my_id: usize) -> Box<dyn lib::Library> {
    return Box::new(DynLib {});
}