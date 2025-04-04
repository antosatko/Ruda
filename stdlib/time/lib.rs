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
        let m = &mut ctx.memory;
        match id {
            // time::time
            0 => {
                let time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs_f64();
                return Ok(Types::Float(time));
            }
            // time::Rng::new
            1 => {
                let rng = Random::new(lib_id);
                let ud = ctx.memory.user_data.push(Box::new(rng));
                return Ok(Types::Pointer(ud, PointerTypes::UserData));
            }
            // time::Rng::coin
            2 => {
                if let Types::Pointer(ud, PointerTypes::UserData) = m.args()[0] {
                    let ud = &mut ctx.memory.user_data.data[ud];
                    let ud = Random::from_ud(ud.as_mut())?;
                    return Ok(Types::Bool(ud.rng.gen()));
                }
            }
            // time::Rng::range
            3 => {
                if let Types::Pointer(ud, PointerTypes::UserData) = m.args()[0] {
                    let min = match m.args()[1] {
                        Types::Int(i) => i,
                        _ => return Err(runtime_error::ErrTypes::Message("Invalid type".to_owned())),
                    };
                    let max = match m.args()[2] {
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
                if let Types::Pointer(ud, PointerTypes::UserData) = m.args()[0] {
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
                if let Types::Pointer(ud, PointerTypes::UserData) = m.args()[0] {
                    let ud = ctx.memory.user_data.data[ud].as_mut();
                    let ud = Clock::from_ud(ud)?;
                    ud.note = std::time::Instant::now();
                    return Ok(Types::Void);
                }
            }
            // time::Clock::elapsed
            7 => {
                if let Types::Pointer(ud, PointerTypes::UserData) = m.args()[0] {
                    let ud = ctx.memory.user_data.data[ud].as_mut();
                    let ud = Clock::from_ud(ud)?;
                    let elapsed = ud.note.elapsed().as_secs_f64();
                    return Ok(Types::Float(elapsed));
                }
            }
            // time::sleep
            8 => {
                let ms = match m.args()[0] {
                    Types::Float(i) => i,
                    _ => return Err(runtime_error::ErrTypes::Message("Invalid type".to_owned())),
                };
                spin_sleep::sleep(std::time::Duration::from_secs_f64(ms));
                return Ok(Types::Void);
            }
            
            _ => unreachable!("Invalid function id {}", id),
        }
        return Ok(runtime_types::Types::Void);
    }


#[no_mangle]
fn register() -> String {
    r#"
    /// Clock UserData: Represents a clock to track time.
    /// This user data type provides functionality for tracking and managing elapsed time.
    userdata Clock > 1i {
        /// Creates a new clock instance.
        new () > 5i

        /// Returns the current time as a float (elapsed time in seconds).
        fun time(): float > 0i

        /// Resets the clock's elapsed time.
        fun reset(self=reg.ptr) > 6i

        /// Returns the elapsed time since the clock was last reset.
        fun elapsed(self=reg.ptr): float > 7i
    }

    /// Rng UserData: Represents a random number generator.
    /// This user data type provides functionality for generating random values.
    userdata Rng > 0i {
        /// Creates a new RNG instance.
        new () > 1i

        /// Generates a random boolean value (coin flip).
        fun coin(self=reg.ptr): bool > 2i

        /// Generates a random integer within the specified range.
        fun range(self=reg.ptr, min=reg.g1: int, max=reg.g2: int): int > 3i

        /// Generates a random floating-point number.
        fun gen(self=reg.ptr): float > 4i
    }


    /// Returns the current system time as a float (elapsed time in seconds).
    fun time(): float > 0i

    /// Pauses the program for a specified number of milliseconds.
    fun sleep(ms=reg.g1: float) > 8i
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