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

use runtime::runtime_error::ErrTypes;
use runtime::runtime_types::*;
use runtime::*;

use runtime::user_data::UserData;
use sfml::*;

fn call(ctx: &mut Context, id: usize, lib_id: usize) -> Result<Types, runtime_error::ErrTypes> {
        let m = &mut ctx.memory;
        match id {
            // Window::new
            0 => {
                let args = m.args();
                let title = match args[0] {
                    Types::Pointer(pos, PointerTypes::String) => m.strings.pool[pos].clone(),
                    _ => Err(ErrTypes::InvalidType(args[0], Types::Pointer(0, PointerTypes::String)))?,
                };
                let settings = match args[1] {
                    Types::Pointer(pos, PointerTypes::UserData) => m.user_data.data[pos].as_mut(),
                    _ => Err(ErrTypes::InvalidType(args[1], Types::Pointer(0, PointerTypes::UserData)))?,
                };
                let settings = settings.as_any_mut().downcast_mut::<WinBuilder>().unwrap();
                let window = Window::new(
                    (settings.width, settings.height),
                    &title,
                    window::Style::CLOSE,
                    settings,
                );
                let ud = ctx.memory.user_data.push(Box::new(window));
                return Ok(Types::Pointer(ud, PointerTypes::UserData));
            }
            // WinBuilder::new
            1 => {
                let ud = ctx.memory.user_data.push(Box::new(WinBuilder::new()));
                return Ok(Types::Pointer(ud, PointerTypes::UserData));
            }
            // WinBuilder::title
            2 => {
                let args = m.args();
                let arg1 = args[1];
                let ud = match args[0] {
                    Types::Pointer(pos, PointerTypes::UserData) => pos,
                    _ => Err(ErrTypes::InvalidType(args[0], Types::Pointer(0, PointerTypes::UserData)))?,
                };
                let win_builder = m.user_data.data[ud].as_mut();
                let win_builder = win_builder.as_any_mut().downcast_mut::<WinBuilder>().unwrap();
                match arg1 {
                    Types::Pointer(pos, PointerTypes::String) => {
                        win_builder.title = m.strings.pool[pos].clone();
                    },
                    Types::Null => (),
                    _ => Err(ErrTypes::InvalidType(arg1, Types::Pointer(0, PointerTypes::String)))?,
                };
                let str_pos = m.strings.from_str(&win_builder.title);
                return Ok(Types::Pointer(str_pos, PointerTypes::String));
            }
            // WinBuilder::width
            3 => {
                let args = m.args();
                let arg1 = args[1];
                let ud = match args[0] {
                    Types::Pointer(pos, PointerTypes::UserData) => pos,
                    _ => Err(ErrTypes::InvalidType(args[0], Types::Pointer(0, PointerTypes::UserData)))?,
                };
                let win_builder = m.user_data.data[ud].as_mut();
                let win_builder = win_builder.as_any_mut().downcast_mut::<WinBuilder>().unwrap();
                match arg1 {
                    Types::Uint(i) => {
                        win_builder.width = i as u32;
                    },
                    Types::Null => (),
                    _ => Err(ErrTypes::InvalidType(arg1, Types::Uint(0)))?,
                };
                return Ok(Types::Uint(win_builder.width as usize));
            }
            // WinBuilder::height
            4 => {
                let args = m.args();
                let arg1 = args[1];
                let ud = match args[0] {
                    Types::Pointer(pos, PointerTypes::UserData) => pos,
                    _ => Err(ErrTypes::InvalidType(args[0], Types::Pointer(0, PointerTypes::UserData)))?,
                };
                let win_builder = m.user_data.data[ud].as_mut();
                let win_builder = win_builder.as_any_mut().downcast_mut::<WinBuilder>().unwrap();
                match arg1 {
                    Types::Uint(i) => {
                        win_builder.height = i as u32;
                    },
                    Types::Null => (),
                    _ => Err(ErrTypes::InvalidType(arg1, Types::Uint(0)))?,
                };
                return Ok(Types::Uint(win_builder.height as usize));
            }
            _ => unreachable!("Invalid function id, {}", id),
        }
        return Ok(runtime_types::Types::Void);
    }


#[no_mangle]
fn register() -> String {
    r#"
    userdata Window > 0i {
        new (title=reg.g1: string, settings=reg.ptr: WinBuilder) > 0i
    }

    userdata WinBuilder > 1i {
        new () > 1i

        fun title(self=reg.ptr, title=reg.g1: string?): string > 2i
        fun width(self=reg.ptr, width=reg.g1: uint?): uint > 3i
        fun height(self=reg.ptr, height=reg.g1: uint?): uint > 4i
    }
    "#.to_string()
}

#[no_mangle]
pub fn init(_ctx: &mut Context, my_id: usize) -> fn(&mut Context, usize, usize) -> Result<Types, runtime_error::ErrTypes> {
    call
}

struct Window {
    window: window::Window,
    name: String,
    id: usize,
    lib_id: usize,
    gc_method: user_data::GcMethod,
}

impl Window {
    fn new(
        size: (u32, u32),
        title: &str,
        style: window::Style,
        settings: &WinBuilder
    ) -> Self {
        let window = window::Window::new(
            size,
            title,
            style,
            &Default::default(),
        );
        Self {
            window,
            name: "Window".to_string(),
            id: 0,
            lib_id: 0,
            gc_method: user_data::GcMethod::None,
        }
    }
}


impl UserData for Window {
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

struct WinBuilder {
    title: String,
    width: u32,
    height: u32,
    name: String,
    id: usize,
    lib_id: usize,
    gc_method: user_data::GcMethod,
}

impl WinBuilder {
    fn new() -> Self {
        Self {
            title: "Window".to_string(),
            width: 800,
            height: 600,
            name: "WinBuilder".to_string(),
            id: 1,
            lib_id: 0,
            gc_method: user_data::GcMethod::None,
        }
    }
}

impl UserData for WinBuilder {
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