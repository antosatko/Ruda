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

use rand::Rng;
use runtime::runtime_types::*;
use runtime::*;
use runtime::user_data::UserData;

fn call(ctx: &mut Context, id: usize, lib_id: usize) -> Result<Types, runtime_error::ErrTypes> {
        let _m = &mut ctx.memory;
        macro_rules! get_args {
            () => {
                match _m.args() {
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
            // time::time
            0 => {
                let time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_millis();
                return Ok(Types::Uint(time as usize));
            }
            // time::Rng::new
            1 => {
                let rng = Random::new(lib_id);
                let ud = ctx.memory.user_data.push(Box::new(rng));
                return Ok(Types::Pointer(ud, PointerTypes::UserData));
            }
            // time::Rng::coin
            2 => {
                if let Types::Pointer(ud, PointerTypes::UserData) = get_args!()[0] {
                    let ud = &mut ctx.memory.user_data.data[ud];
                    let ud = Random::from_ud(ud.as_mut())?;
                    return Ok(Types::Bool(ud.rng.gen()));
                }
            }
            // time::Rng::range
            3 => {
                if let Types::Pointer(ud, PointerTypes::UserData) = get_args!()[0] {
                    let min = match get_args!()[1] {
                        Types::Int(i) => i,
                        _ => return Err(runtime_error::ErrTypes::Message("Invalid type".to_owned())),
                    };
                    let max = match get_args!()[2] {
                        Types::Int(i) => i,
                        _ => return Err(runtime_error::ErrTypes::Message("Invalid type".to_owned())),
                    };
                    let ud = ctx.memory.user_data.data[ud].as_mut();
                    let ud = Random::from_ud(ud)?;
                    return Ok(Types::Int(ud.rng.gen_range(min..max)));
                }
            }
            // time::Rng::gen
            4 => {
                if let Types::Pointer(ud, PointerTypes::UserData) = get_args!()[0] {
                    let ud = ctx.memory.user_data.data[ud].as_mut();
                    let ud = Random::from_ud(ud)?;
                    return Ok(Types::Float(ud.rng.gen()));
                }
            }
            // time::Clock::new
            5 => {
                let clock = Clock::new(lib_id);
                let ud = ctx.memory.user_data.push(Box::new(clock));
                return Ok(Types::Pointer(ud, PointerTypes::UserData));
            }
            // time::Clock::reset
            6 => {
                if let Types::Pointer(ud, PointerTypes::UserData) = get_args!()[0] {
                    let ud = ctx.memory.user_data.data[ud].as_mut();
                    let ud = Clock::from_ud(ud)?;
                    ud.note = std::time::Instant::now();
                    return Ok(Types::Void);
                }
            }
            // time::Clock::elapsed
            7 => {
                if let Types::Pointer(ud, PointerTypes::UserData) = get_args!()[0] {
                    let ud = ctx.memory.user_data.data[ud].as_mut();
                    let ud = Clock::from_ud(ud)?;
                    let elapsed = ud.note.elapsed().as_millis();
                    return Ok(Types::Uint(elapsed as usize));
                }
            }
            
            _ => unreachable!("Invalid function id"),
        }
        return Ok(runtime_types::Types::Void);
    }


#[no_mangle]
fn register() -> String {
    r#"
    userdata Clock > 1i {
        new () > 5i

        fun time(): uint > 0i
        fun reset(self=reg.ptr) > 6i
        fun elapsed(self=reg.ptr): uint > 7i
    }

    userdata Rng > 0i {
        new () > 1i

        fun coin(self=reg.ptr): bool > 2i
        fun range(self=reg.ptr, min=reg.g1: int, max=reg.g2: int): int > 3i
        fun gen(self=reg.ptr): float > 4i
    }
    fun time(): uint > 0i
    "#.to_string()
}

#[no_mangle]
pub fn init(_ctx: &mut Context, my_id: usize) -> fn(&mut Context, usize, usize) -> Result<Types, runtime_error::ErrTypes> {
    call
}


struct Random {
    rng: rand::rngs::ThreadRng,
    id: usize,
    lib_id: usize,
    gc_method: user_data::GcMethod,
    name: String,
}

impl Random {
    const ASSIGN_ID: usize = 0;

    fn new(lib_id: usize) -> Self {
        Self {
            rng: rand::thread_rng(),
            lib_id,
            name: "Rng".to_owned(),
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

impl UserData for Random {
    fn label(&self) -> &str {
        &self.name
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

struct Clock {
    note: std::time::Instant,
    id: usize,
    lib_id: usize,
    gc_method: user_data::GcMethod,
    name: String,
}

impl Clock {
    const ASSIGN_ID: usize = 1;

    fn new(lib_id: usize) -> Self {
        Self {
            note: std::time::Instant::now(),
            lib_id,
            name: "Clock".to_owned(),
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

impl UserData for Clock {
    fn label(&self) -> &str {
        &self.name
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