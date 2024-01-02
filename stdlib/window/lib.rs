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

use std::sync::Arc;

use runtime::runtime_error::ErrTypes;
use runtime::runtime_types::*;
use runtime::*;

use runtime::user_data::UserData;
use sfml::graphics::{
    CircleShape, Color, Font, Image, RectangleShape, RenderTarget, RenderWindow, Shape, Text,
    Transformable, Sprite, VertexBufferUsage,
};
use sfml::system::{Vector2, Vector2f};
use sfml::window::Style;
use sfml::*;

fn call(ctx: &mut Context, id: usize, lib_id: usize) -> Result<Types, runtime_error::ErrTypes> {
    let m = &mut ctx.memory;
    match id {
        // Window::new
        0 => {
            let args = m.args();
            let title = match args[0] {
                Types::Pointer(pos, PointerTypes::String) => m.strings.pool[pos].clone(),
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::String),
                ))?,
            };
            let settings = match args[1] {
                Types::Pointer(pos, PointerTypes::UserData) => m.user_data.data[pos].as_mut(),
                _ => Err(ErrTypes::InvalidType(
                    args[1],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let settings = settings.as_any_mut().downcast_mut::<WinBuilder>().unwrap();
            let mut window = Window::new(
                (settings.width, settings.height),
                &title,
                settings.style,
                settings,
            );
            match Image::from_memory(include_bytes!("../../logo.png")) {
                Some(icon) => {
                    unsafe {
                        window
                            .window
                            .set_icon(icon.size().x, icon.size().y, icon.pixel_data())
                    };
                }
                None => (),
            };
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
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let win_builder = m.user_data.data[ud].as_mut();
            let win_builder = win_builder
                .as_any_mut()
                .downcast_mut::<WinBuilder>()
                .unwrap();
            match arg1 {
                Types::Pointer(pos, PointerTypes::String) => {
                    win_builder.title = m.strings.pool[pos].clone();
                }
                _ => Err(ErrTypes::InvalidType(
                    arg1,
                    Types::Pointer(0, PointerTypes::String),
                ))?,
            };
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // WinBuilder::width
        3 => {
            let args = m.args();
            let arg1 = args[1];
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let win_builder = m.user_data.data[ud].as_mut();
            let win_builder = win_builder
                .as_any_mut()
                .downcast_mut::<WinBuilder>()
                .unwrap();
            match arg1 {
                Types::Uint(i) => {
                    win_builder.width = i as u32;
                }
                _ => Err(ErrTypes::InvalidType(arg1, Types::Uint(0)))?,
            };
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // WinBuilder::height
        4 => {
            let args = m.args();
            let arg1 = args[1];
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let win_builder = m.user_data.data[ud].as_mut();
            let win_builder = win_builder
                .as_any_mut()
                .downcast_mut::<WinBuilder>()
                .unwrap();
            match arg1 {
                Types::Uint(i) => {
                    win_builder.height = i as u32;
                }
                _ => Err(ErrTypes::InvalidType(arg1, Types::Uint(0)))?,
            };
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // Window::clear
        5 => {
            let args = m.args();
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            window.window.clear(window.bg);
        }
        // Window::display
        6 => {
            let args = m.args();
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            window.window.display();
        }
        // Window::close
        7 => {
            let args = m.args();
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            window.window.close();
        }
        // Window::width
        8 => {
            let args = m.args();
            let arg1 = args[1];
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            match arg1 {
                Types::Uint(i) => {
                    window.window.set_size((i as u32, window.window.size().x));
                }
                Types::Null => (),
                _ => Err(ErrTypes::InvalidType(arg1, Types::Uint(0)))?,
            };
            return Ok(Types::Uint(window.window.size().x as usize));
        }
        // Window::height
        9 => {
            let args = m.args();
            let arg1 = args[1];
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            match arg1 {
                Types::Uint(i) => {
                    window.window.set_size((window.window.size().x, i as u32));
                }
                Types::Null => (),
                _ => Err(ErrTypes::InvalidType(arg1, Types::Uint(0)))?,
            };
            return Ok(Types::Uint(window.window.size().y as usize));
        }
        // Window::pollEvent
        10 => {
            let args = m.args();
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            let event = match window.window.poll_event() {
                Some(e) => e,
                None => return Ok(Types::Null),
            };
            let ud = ctx.memory.user_data.push(Box::new(Event::new(event)));
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // Event::code
        11 => {
            let args = m.args();
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let event = m.user_data.data[ud].as_mut();
            let event = event.as_any_mut().downcast_mut::<Event>().unwrap();
            let code = match event.event {
                window::Event::Closed => 0,
                window::Event::Resized { .. } => 1,
                window::Event::LostFocus => 2,
                window::Event::GainedFocus => 3,
                window::Event::TextEntered { .. } => 4,
                window::Event::KeyPressed { .. } => 5,
                window::Event::KeyReleased { .. } => 6,
                window::Event::MouseWheelScrolled { .. } => 7,
                window::Event::MouseButtonPressed { .. } => 8,
                window::Event::MouseButtonReleased { .. } => 9,
                window::Event::MouseMoved { .. } => 10,
                _ => 11,
            };
            return Ok(Types::Uint(code));
        }
        // Event::key
        12 => {
            let args = m.args();
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let event = m.user_data.data[ud].as_mut();
            let event = event.as_any_mut().downcast_mut::<Event>().unwrap();
            let key = match event.event {
                window::Event::KeyPressed { code, .. } => code as usize,
                window::Event::KeyReleased { code, .. } => code as usize,
                _ => 0,
            };
            return Ok(Types::Uint(key as usize));
        }
        // Window::fps
        13 => {
            let args = m.args();
            let arg1 = args[1];
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            match arg1 {
                Types::Uint(i) => {
                    window.window.set_framerate_limit(i as u32);
                }
                _ => Err(ErrTypes::InvalidType(arg1, Types::Uint(0)))?,
            };
            return Ok(Types::Void);
        }
        // Event::input
        14 => {
            let args = m.args();
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        args[0],
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };
            let event = m.user_data.data[ud].as_mut();
            let event = event.as_any_mut().downcast_mut::<Event>().unwrap();
            let input = match event.event {
                window::Event::TextEntered { unicode } => unicode as char,
                _ => '\0',
            };
            return Ok(Types::Char(input));
        }
        // Window::title
        15 => {
            let args = m.args();
            let arg1 = args[1];
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        args[0],
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            match arg1 {
                Types::Pointer(pos, PointerTypes::String) => {
                    window.window.set_title(&m.strings.pool[pos]);
                }
                _ => {
                    return Err(ErrTypes::InvalidType(
                        arg1,
                        Types::Pointer(0, PointerTypes::String),
                    ))
                }
            };
            return Ok(Types::Void);
        }
        // Event::scan
        16 => {
            let args = m.args();
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        args[0],
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };
            let event = m.user_data.data[ud].as_mut();
            let event = event.as_any_mut().downcast_mut::<Event>().unwrap();
            let scan = match event.event {
                window::Event::KeyPressed { scan, .. } => scan as usize,
                _ => 0,
            };
            return Ok(Types::Uint(scan as usize));
        }
        // Event::verticalWheel
        17 => {
            let args = m.args();
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        args[0],
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };
            let event = m.user_data.data[ud].as_mut();
            let event = event.as_any_mut().downcast_mut::<Event>().unwrap();
            let wheel = match event.event {
                window::Event::MouseWheelScrolled { delta, wheel, .. } => match wheel {
                    window::mouse::Wheel::VerticalWheel => delta as f64,
                    window::mouse::Wheel::HorizontalWheel => 0.0,
                },
                _ => 0.0,
            };
            return Ok(Types::Float(wheel));
        }
        // Event::horizontalWheel
        18 => {
            let args = m.args();
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        args[0],
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };
            let event = m.user_data.data[ud].as_mut();
            let event = event.as_any_mut().downcast_mut::<Event>().unwrap();
            let wheel = match event.event {
                window::Event::MouseWheelScrolled { delta, wheel, .. } => match wheel {
                    window::mouse::Wheel::VerticalWheel => 0.0,
                    window::mouse::Wheel::HorizontalWheel => delta as f64,
                },
                _ => 0.0,
            };
            return Ok(Types::Float(wheel));
        }
        // Event::mouseX
        19 => {
            let args = m.args();
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        args[0],
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };
            let event = m.user_data.data[ud].as_mut();
            let event = event.as_any_mut().downcast_mut::<Event>().unwrap();
            let x = match event.event {
                window::Event::MouseMoved { x, .. } => x as usize,
                window::Event::MouseButtonPressed { x, .. } => x as usize,
                window::Event::MouseButtonReleased { x, .. } => x as usize,
                _ => 0,
            };
            return Ok(Types::Uint(x));
        }
        // Event::mouseY
        20 => {
            let args = m.args();
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        args[0],
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };
            let event = m.user_data.data[ud].as_mut();
            let event = event.as_any_mut().downcast_mut::<Event>().unwrap();
            let y = match event.event {
                window::Event::MouseMoved { y, .. } => y as usize,
                window::Event::MouseButtonPressed { y, .. } => y as usize,
                window::Event::MouseButtonReleased { y, .. } => y as usize,
                _ => 0,
            };
            return Ok(Types::Uint(y));
        }
        // Event::mouseButton
        21 => {
            let args = m.args();
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        args[0],
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };
            let event = m.user_data.data[ud].as_mut();
            let event = event.as_any_mut().downcast_mut::<Event>().unwrap();
            let button = match event.event {
                window::Event::MouseButtonPressed { button, .. } => button as usize,
                window::Event::MouseButtonReleased { button, .. } => button as usize,
                _ => 0,
            };
            return Ok(Types::Uint(button));
        }
        // Event::alt
        22 => {
            let args = m.args();
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        args[0],
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };
            let event = m.user_data.data[ud].as_mut();
            let event = event.as_any_mut().downcast_mut::<Event>().unwrap();
            let alt = match event.event {
                window::Event::KeyPressed { alt, .. } => alt,
                window::Event::KeyReleased { alt, .. } => alt,
                _ => false,
            };
            return Ok(Types::Bool(alt));
        }
        // Event::control
        23 => {
            let args = m.args();
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        args[0],
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };
            let event = m.user_data.data[ud].as_mut();
            let event = event.as_any_mut().downcast_mut::<Event>().unwrap();
            let control = match event.event {
                window::Event::KeyPressed { ctrl, .. } => ctrl,
                window::Event::KeyReleased { ctrl, .. } => ctrl,
                _ => false,
            };
            return Ok(Types::Bool(control));
        }
        // Event::shift
        24 => {
            let args = m.args();
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        args[0],
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };
            let event = m.user_data.data[ud].as_mut();
            let event = event.as_any_mut().downcast_mut::<Event>().unwrap();
            let shift = match event.event {
                window::Event::KeyPressed { shift, .. } => shift,
                window::Event::KeyReleased { shift, .. } => shift,
                _ => false,
            };
            return Ok(Types::Bool(shift));
        }
        // Event::system
        25 => {
            let args = m.args();
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        args[0],
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };
            let event = m.user_data.data[ud].as_mut();
            let event = event.as_any_mut().downcast_mut::<Event>().unwrap();
            let system = match event.event {
                window::Event::KeyPressed { system, .. } => system,
                window::Event::KeyReleased { system, .. } => system,
                _ => false,
            };
            return Ok(Types::Bool(system));
        }
        // DrawStyle::new
        26 => {
            let ud = ctx.memory.user_data.push(Box::new(DrawStyle::new()));
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // DrawStyle::color
        27 => {
            let args = m.args();
            let arg1 = args[1];
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        args[0],
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };
            let color = match arg1 {
                Types::Pointer(pos, PointerTypes::UserData) => {
                    let color = m.user_data.data[pos].as_mut();
                    let color = color.as_any_mut().downcast_mut::<WinColor>().unwrap();
                    color.clone()
                }
                _ => {
                    return Err(ErrTypes::InvalidType(
                        arg1,
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };

            let draw_style = m.user_data.data[ud].as_mut();
            let draw_style = draw_style.as_any_mut().downcast_mut::<DrawStyle>().unwrap();
            draw_style.color = Color {
                r: color.r,
                g: color.g,
                b: color.b,
                a: color.a,
            };
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // Color::new
        28 => {
            let args = m.args();
            let r = match args[0] {
                Types::Uint(i) => i as u8,
                _ => return Err(ErrTypes::InvalidType(args[0], Types::Uint(0))),
            };
            let g = match args[1] {
                Types::Uint(i) => i as u8,
                _ => return Err(ErrTypes::InvalidType(args[1], Types::Uint(0))),
            };
            let b = match args[2] {
                Types::Uint(i) => i as u8,
                _ => return Err(ErrTypes::InvalidType(args[2], Types::Uint(0))),
            };
            let a = match args[3] {
                Types::Uint(i) => i as u8,
                _ => return Err(ErrTypes::InvalidType(args[3], Types::Uint(0))),
            };
            let color = WinColor::new(r, g, b, a);
            let ud = ctx.memory.user_data.push(Box::new(color));
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // Color::r
        29 => {
            let args = m.args();
            let arg1 = args[1];
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        args[0],
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };
            let color = m.user_data.data[ud].as_mut();
            let color = color.as_any_mut().downcast_mut::<WinColor>().unwrap();
            match arg1 {
                Types::Uint(i) => {
                    color.r = i as u8;
                }
                Types::Null => (),
                _ => return Err(ErrTypes::InvalidType(arg1, Types::Uint(0))),
            };
            return Ok(Types::Uint(color.r as usize));
        }
        // Color::g
        30 => {
            let args = m.args();
            let arg1 = args[1];
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        args[0],
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };
            let color = m.user_data.data[ud].as_mut();
            let color = color.as_any_mut().downcast_mut::<WinColor>().unwrap();
            match arg1 {
                Types::Uint(i) => {
                    color.g = i as u8;
                }
                Types::Null => (),
                _ => return Err(ErrTypes::InvalidType(arg1, Types::Uint(0))),
            };
            return Ok(Types::Uint(color.g as usize));
        }
        // Color::b
        31 => {
            let args = m.args();
            let arg1 = args[1];
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        args[0],
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };
            let color = m.user_data.data[ud].as_mut();
            let color = color.as_any_mut().downcast_mut::<WinColor>().unwrap();
            match arg1 {
                Types::Uint(i) => {
                    color.b = i as u8;
                }
                Types::Null => (),
                _ => return Err(ErrTypes::InvalidType(arg1, Types::Uint(0))),
            };
            return Ok(Types::Uint(color.b as usize));
        }
        // Color::a
        32 => {
            let args = m.args();
            let arg1 = args[1];
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        args[0],
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };
            let color = m.user_data.data[ud].as_mut();
            let color = color.as_any_mut().downcast_mut::<WinColor>().unwrap();
            match arg1 {
                Types::Uint(i) => {
                    color.a = i as u8;
                }
                Types::Null => (),
                _ => return Err(ErrTypes::InvalidType(arg1, Types::Uint(0))),
            };
            return Ok(Types::Uint(color.a as usize));
        }
        // Color::fromHex
        33 => {
            let args = m.args();
            let arg1 = args[0];
            let hex = match arg1 {
                Types::Pointer(pos, PointerTypes::String) => &m.strings.pool[pos],
                _ => {
                    return Err(ErrTypes::InvalidType(
                        arg1,
                        Types::Pointer(0, PointerTypes::String),
                    ))
                }
            };
            let color = WinColor::from_hex(hex);
            let ud = ctx.memory.user_data.push(Box::new(color));
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // Color::copy
        34 => {
            let args = m.args();
            let arg1 = args[0];
            let ud = match arg1 {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        arg1,
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };
            let color = m.user_data.data[ud].as_mut();
            let color = color.as_any_mut().downcast_mut::<WinColor>().unwrap();
            let color = color.clone();
            let ud = ctx.memory.user_data.push(Box::new(color));
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // DrawStyle::rotation
        35 => {
            let args = m.args();
            let arg1 = args[1];
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        args[0],
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };
            let rotation = match arg1 {
                Types::Float(f) => f as f32,
                _ => return Err(ErrTypes::InvalidType(arg1, Types::Float(0.0))),
            };
            let draw_style = m.user_data.data[ud].as_mut();
            let draw_style = draw_style.as_any_mut().downcast_mut::<DrawStyle>().unwrap();
            draw_style.rotation = rotation;
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // DrawStyle::scaleX
        36 => {
            let args = m.args();
            let arg1 = args[1];
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        args[0],
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };
            let scale_x = match arg1 {
                Types::Float(f) => f as f32,
                _ => return Err(ErrTypes::InvalidType(arg1, Types::Float(0.0))),
            };
            let draw_style = m.user_data.data[ud].as_mut();
            let draw_style = draw_style.as_any_mut().downcast_mut::<DrawStyle>().unwrap();
            draw_style.scale.0 = scale_x;
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // DrawStyle::scaleY
        37 => {
            let args = m.args();
            let arg1 = args[1];
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        args[0],
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };
            let scale_y = match arg1 {
                Types::Float(f) => f as f32,
                _ => return Err(ErrTypes::InvalidType(arg1, Types::Float(0.0))),
            };
            let draw_style = m.user_data.data[ud].as_mut();
            let draw_style = draw_style.as_any_mut().downcast_mut::<DrawStyle>().unwrap();
            draw_style.scale.1 = scale_y;
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // DrawStyle::outlineColor
        38 => {
            let args = m.args();
            let arg1 = args[1];
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        args[0],
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };
            let color = match arg1 {
                Types::Pointer(pos, PointerTypes::UserData) => {
                    let color = m.user_data.data[pos].as_mut();
                    let color = color.as_any_mut().downcast_mut::<WinColor>().unwrap();
                    color.clone()
                }
                _ => {
                    return Err(ErrTypes::InvalidType(
                        arg1,
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };

            let draw_style = m.user_data.data[ud].as_mut();
            let draw_style = draw_style.as_any_mut().downcast_mut::<DrawStyle>().unwrap();
            draw_style.outline_color = Color {
                r: color.r,
                g: color.g,
                b: color.b,
                a: color.a,
            };
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // DrawStyle::outlineThickness
        39 => {
            let args = m.args();
            let arg1 = args[1];
            let thickness = match arg1 {
                Types::Float(f) => f as f32,
                _ => return Err(ErrTypes::InvalidType(arg1, Types::Float(0.0))),
            };
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        args[0],
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };

            let draw_style = m.user_data.data[ud].as_mut();
            let draw_style = draw_style.as_any_mut().downcast_mut::<DrawStyle>().unwrap();
            draw_style.outline_thickness = thickness;
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // DrawStyle::font
        40 => {
            let args = m.args();
            let arg1 = args[1];
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        args[0],
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };
            let font = match arg1 {
                Types::Pointer(pos, PointerTypes::UserData) => {
                    let font = m.user_data.data[pos].as_mut();
                    font.as_any_mut()
                        .downcast_mut::<WinFont>()
                        .unwrap()
                        .path
                        .clone()
                }
                _ => {
                    return Err(ErrTypes::InvalidType(
                        arg1,
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };

            let draw_style = m.user_data.data[ud].as_mut();
            let draw_style = draw_style.as_any_mut().downcast_mut::<DrawStyle>().unwrap();
            match Font::from_file(&font) {
                Some(font) => draw_style.font = Some(Arc::new(font)),
                None => (),
            };
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // DrawStyle::fontSize
        41 => {
            let args = m.args();
            let arg1 = args[1];
            let size = match arg1 {
                Types::Uint(i) => i as u32,
                _ => return Err(ErrTypes::InvalidType(arg1, Types::Uint(0))),
            };
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        args[0],
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };

            let draw_style = m.user_data.data[ud].as_mut();
            let draw_style = draw_style.as_any_mut().downcast_mut::<DrawStyle>().unwrap();
            draw_style.font_size = size;
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // Font::new
        42 => {
            let args = m.args();
            let path = match args[0] {
                Types::Pointer(pos, PointerTypes::String) => &m.strings.pool[pos],
                _ => {
                    return Err(ErrTypes::InvalidType(
                        args[0],
                        Types::Pointer(0, PointerTypes::String),
                    ))
                }
            };
            let font = WinFont::new(path);
            let ud = ctx.memory.user_data.push(Box::new(font));
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // Font::default
        43 => {
            let args = m.args();
            let name = match args[0] {
                Types::Pointer(pos, PointerTypes::String) => &m.strings.pool[pos],
                _ => {
                    return Err(ErrTypes::InvalidType(
                        args[0],
                        Types::Pointer(0, PointerTypes::String),
                    ))
                }
            };
            let font = WinFont::default(name);
            let ud = ctx.memory.user_data.push(Box::new(font));
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // Font::ubuntuMono
        44 => {
            let font = WinFont::ubuntu_mono();
            let ud = ctx.memory.user_data.push(Box::new(font));
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // Font::roboto
        45 => {
            let font = WinFont::roboto();
            let ud = ctx.memory.user_data.push(Box::new(font));
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // Color::from
        46 => {
            let args = m.args();
            let color = match args[0] {
                Types::Uint(i) => match i {
                    0 => Color::BLACK,
                    1 => Color::WHITE,
                    2 => Color::RED,
                    3 => Color::GREEN,
                    4 => Color::BLUE,
                    5 => Color::YELLOW,
                    6 => Color::MAGENTA,
                    7 => Color::CYAN,
                    8 => Color::TRANSPARENT,
                    _ => return Err(ErrTypes::InvalidType(args[0], Types::Uint(0))),
                },
                _ => return Err(ErrTypes::InvalidType(args[0], Types::Uint(0))),
            };
            let win_color =
                WinColor::new(color.r as u8, color.g as u8, color.b as u8, color.a as u8);
            let ud = ctx.memory.user_data.push(Box::new(win_color));
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // Window::drawRectangle
        47 => {
            let args = m.args();
            let arg1 = args[1];
            let arg2 = args[2];
            let arg3 = args[3];
            let arg4 = args[4];
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let x = match arg1 {
                Types::Float(f) => f as f32,
                _ => Err(ErrTypes::InvalidType(arg1, Types::Float(0.0)))?,
            };
            let y = match arg2 {
                Types::Float(f) => f as f32,
                _ => Err(ErrTypes::InvalidType(arg2, Types::Float(0.0)))?,
            };
            let width = match arg3 {
                Types::Float(f) => f as f32,
                _ => Err(ErrTypes::InvalidType(arg3, Types::Float(0.0)))?,
            };
            let height = match arg4 {
                Types::Float(f) => f as f32,
                _ => Err(ErrTypes::InvalidType(arg4, Types::Float(0.0)))?,
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            let mut rect = RectangleShape::new();
            rect.set_position((x, y));
            rect.set_size((width, height));
            rect.set_fill_color(window.style.color);
            rect.set_outline_color(window.style.outline_color);
            rect.set_outline_thickness(window.style.outline_thickness);
            rect.set_rotation(window.style.rotation);
            rect.set_scale((window.style.scale.0, window.style.scale.1));
            rect.set_origin(window.style.origin);
            window.window.draw(&rect);
            return Ok(Types::Void);
        }
        // Window::drawCircle
        48 => {
            let args = m.args();
            let arg1 = args[1];
            let arg2 = args[2];
            let arg3 = args[3];
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let x = match arg1 {
                Types::Float(f) => f as f32,
                _ => Err(ErrTypes::InvalidType(arg1, Types::Float(0.0)))?,
            };
            let y = match arg2 {
                Types::Float(f) => f as f32,
                _ => Err(ErrTypes::InvalidType(arg2, Types::Float(0.0)))?,
            };
            let radius = match arg3 {
                Types::Float(f) => f as f32,
                _ => Err(ErrTypes::InvalidType(arg3, Types::Float(0.0)))?,
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            let mut circle = CircleShape::new(radius, 30);
            circle.set_position((x, y));
            circle.set_fill_color(window.style.color);
            circle.set_outline_color(window.style.outline_color);
            circle.set_outline_thickness(window.style.outline_thickness);
            circle.set_rotation(window.style.rotation);
            circle.set_scale((window.style.scale.0, window.style.scale.1));
            circle.set_origin(window.style.origin);
            window.window.draw(&circle);
            return Ok(Types::Void);
        }
        // Window::drawText
        49 => {
            let args = m.args();
            let arg1 = args[1];
            let arg2 = args[2];
            let arg3 = args[3];
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let x = match arg1 {
                Types::Float(f) => f as f32,
                _ => Err(ErrTypes::InvalidType(arg1, Types::Float(0.0)))?,
            };
            let y = match arg2 {
                Types::Float(f) => f as f32,
                _ => Err(ErrTypes::InvalidType(arg2, Types::Float(0.0)))?,
            };
            let text = match arg3 {
                Types::Pointer(pos, PointerTypes::String) => &m.strings.pool[pos],
                _ => Err(ErrTypes::InvalidType(
                    arg3,
                    Types::Pointer(0, PointerTypes::String),
                ))?,
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            let style = &window.style;
            let font = match &style.font {
                Some(font) => font,
                None => {
                    return Err(ErrTypes::Message(
                        "Window::drawText: no font set".to_string(),
                    ))
                }
            };
            let mut text = Text::new(text, &font, style.font_size);
            text.set_position((x, y));
            text.set_fill_color(style.color);
            text.set_outline_color(style.color);
            text.set_outline_thickness(style.outline_thickness);
            text.set_rotation(style.rotation);
            text.set_scale((style.scale.0, style.scale.1));
            text.set_letter_spacing(style.character_spacing);
            text.set_line_spacing(style.line_spacing);
            text.set_origin(style.origin);
            window.window.draw(&text);
            return Ok(Types::Void);
        }
        // Window::styledRectangle
        50 => {
            let args = m.args();
            let arg1 = args[1];
            let arg2 = args[2];
            let arg3 = args[3];
            let arg4 = args[4];
            let arg5 = args[5];
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let x = match arg1 {
                Types::Float(f) => f as f32,
                _ => Err(ErrTypes::InvalidType(arg1, Types::Float(0.0)))?,
            };
            let y = match arg2 {
                Types::Float(f) => f as f32,
                _ => Err(ErrTypes::InvalidType(arg2, Types::Float(0.0)))?,
            };
            let width = match arg3 {
                Types::Float(f) => f as f32,
                _ => Err(ErrTypes::InvalidType(arg3, Types::Float(0.0)))?,
            };
            let height = match arg4 {
                Types::Float(f) => f as f32,
                _ => Err(ErrTypes::InvalidType(arg4, Types::Float(0.0)))?,
            };
            let style = match arg5 {
                Types::Pointer(pos, PointerTypes::UserData) => {
                    let style = m.user_data.data[pos].as_mut();
                    style.as_any_mut().downcast_mut::<DrawStyle>().unwrap()
                }
                _ => Err(ErrTypes::InvalidType(
                    arg5,
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let mut rect = RectangleShape::new();
            rect.set_position((x, y));
            rect.set_size((width, height));
            rect.set_fill_color(style.color);
            rect.set_outline_color(style.outline_color);
            rect.set_outline_thickness(style.outline_thickness);
            rect.set_rotation(style.rotation);
            rect.set_scale((style.scale.0, style.scale.1));
            rect.set_origin(style.origin);
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            window.window.draw(&rect);
            return Ok(Types::Void);
        }
        // Window::styledCircle
        51 => {
            let args = m.args();
            let arg1 = args[1];
            let arg2 = args[2];
            let arg3 = args[3];
            let arg4 = args[4];
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let x = match arg1 {
                Types::Float(f) => f as f32,
                _ => Err(ErrTypes::InvalidType(arg1, Types::Float(0.0)))?,
            };
            let y = match arg2 {
                Types::Float(f) => f as f32,
                _ => Err(ErrTypes::InvalidType(arg2, Types::Float(0.0)))?,
            };
            let radius = match arg3 {
                Types::Float(f) => f as f32,
                _ => Err(ErrTypes::InvalidType(arg3, Types::Float(0.0)))?,
            };
            let style = match arg4 {
                Types::Pointer(pos, PointerTypes::UserData) => {
                    let style = m.user_data.data[pos].as_mut();
                    style.as_any_mut().downcast_mut::<DrawStyle>().unwrap()
                }
                _ => Err(ErrTypes::InvalidType(
                    arg4,
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let mut circle = CircleShape::new(radius, 30);
            circle.set_position((x, y));
            circle.set_fill_color(style.color);
            circle.set_outline_color(style.outline_color);
            circle.set_outline_thickness(style.outline_thickness);
            circle.set_rotation(style.rotation);
            circle.set_scale((style.scale.0, style.scale.1));
            circle.set_origin(style.origin);
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            window.window.draw(&circle);
            return Ok(Types::Void);
        }
        // DrawStyle::characterSpacing
        52 => {
            let args = m.args();
            let arg1 = args[1];
            let spacing = match arg1 {
                Types::Float(f) => f as f32,
                _ => return Err(ErrTypes::InvalidType(arg1, Types::Float(0.0))),
            };
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        args[0],
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };

            let draw_style = m.user_data.data[ud].as_mut();
            let draw_style = draw_style.as_any_mut().downcast_mut::<DrawStyle>().unwrap();
            draw_style.character_spacing = spacing;
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // DrawStyle::lineSpacing
        53 => {
            let args = m.args();
            let arg1 = args[1];
            let spacing = match arg1 {
                Types::Float(f) => f as f32,
                _ => return Err(ErrTypes::InvalidType(arg1, Types::Float(0.0))),
            };
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        args[0],
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };

            let draw_style = m.user_data.data[ud].as_mut();
            let draw_style = draw_style.as_any_mut().downcast_mut::<DrawStyle>().unwrap();
            draw_style.line_spacing = spacing;
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // WinBuilder::build
        54 => {
            let args = m.args();
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let builder = m.user_data.data[ud].as_mut();
            let settings = builder.as_any_mut().downcast_mut::<WinBuilder>().unwrap();
            let mut window = Window::new(
                (settings.width, settings.height),
                &settings.title.clone(),
                settings.style,
                &settings,
            );
            match Image::from_memory(include_bytes!("../../logo.png")) {
                Some(icon) => {
                    unsafe {
                        window
                            .window
                            .set_icon(icon.size().x, icon.size().y, icon.pixel_data())
                    };
                }
                None => (),
            }
            let ud = ctx.memory.user_data.push(Box::new(window));
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // WinBuilder::resize
        55 => {
            let args = m.args();
            let resize = Style::RESIZE;
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };

            let builder = m.user_data.data[ud].as_mut();
            let builder = builder.as_any_mut().downcast_mut::<WinBuilder>().unwrap();
            builder.style = builder.style | resize;
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // WinBuilder::fullscreen
        56 => {
            let args = m.args();
            let fullscreen = Style::FULLSCREEN;
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };

            let builder = m.user_data.data[ud].as_mut();
            let builder = builder.as_any_mut().downcast_mut::<WinBuilder>().unwrap();
            builder.style = builder.style | fullscreen;
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // WinBuilder::close
        57 => {
            let args = m.args();
            let close = Style::CLOSE;
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };

            let builder = m.user_data.data[ud].as_mut();
            let builder = builder.as_any_mut().downcast_mut::<WinBuilder>().unwrap();
            builder.style = builder.style | close;
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // WinBuilder::titlebar
        58 => {
            let args = m.args();
            let titlebar = Style::TITLEBAR;
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };

            let builder = m.user_data.data[ud].as_mut();
            let builder = builder.as_any_mut().downcast_mut::<WinBuilder>().unwrap();
            builder.style = builder.style | titlebar;
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // WinBuilder::default
        59 => {
            let args = m.args();
            let default = Style::DEFAULT;
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };

            let builder = m.user_data.data[ud].as_mut();
            let builder = builder.as_any_mut().downcast_mut::<WinBuilder>().unwrap();
            builder.style = builder.style | default;
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // Window::background
        60 => {
            let args = m.args();
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let arg1 = args[1];
            let color = match arg1 {
                Types::Pointer(pos, PointerTypes::UserData) => {
                    let color = m.user_data.data[pos].as_mut();
                    let color = color.as_any_mut().downcast_mut::<WinColor>().unwrap();
                    color.clone()
                }
                _ => Err(ErrTypes::InvalidType(
                    arg1,
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            window.bg = Color {
                r: color.r,
                g: color.g,
                b: color.b,
                a: color.a,
            };
            return Ok(Types::Void);
        }
        // Window::getStyle
        61 => {
            let args = m.args();
            let arg1 = args[0];
            let ud = match arg1 {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    arg1,
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            let style = window.style.clone();
            let ud = ctx.memory.user_data.push(Box::new(style));
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // Window::setStyle
        62 => {
            let args = m.args();
            let arg1 = args[0];
            let arg2 = args[1];
            let ud = match arg1 {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    arg1,
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let style = match arg2 {
                Types::Pointer(pos, PointerTypes::UserData) => {
                    let style = m.user_data.data[pos].as_mut();
                    style
                        .as_any_mut()
                        .downcast_mut::<DrawStyle>()
                        .unwrap()
                        .clone()
                }
                _ => Err(ErrTypes::InvalidType(
                    arg2,
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            window.style = style;
            return Ok(Types::Void);
        }
        // Window::color
        63 => {
            let args = m.args();
            let arg1 = args[0];
            let arg2 = args[1];
            let ud = match arg1 {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    arg1,
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let color = match arg2 {
                Types::Pointer(pos, PointerTypes::UserData) => {
                    let color = m.user_data.data[pos].as_mut();
                    let color = color.as_any_mut().downcast_mut::<WinColor>().unwrap();
                    color.clone()
                }
                _ => Err(ErrTypes::InvalidType(
                    arg2,
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            window.style.color = Color {
                r: color.r,
                g: color.g,
                b: color.b,
                a: color.a,
            };
            return Ok(Types::Void);
        }
        // Window::rotation
        64 => {
            let args = m.args();
            let arg1 = args[0];
            let arg2 = args[1];
            let rotation = match arg2 {
                Types::Float(f) => f as f32,
                _ => return Err(ErrTypes::InvalidType(arg2, Types::Float(0.0))),
            };
            let ud = match arg1 {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    arg1,
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            window.style.rotation = rotation;
            return Ok(Types::Void);
        }
        // Window::scaleX
        65 => {
            let args = m.args();
            let arg1 = args[0];
            let arg2 = args[1];
            let scale_x = match arg2 {
                Types::Float(f) => f as f32,
                _ => return Err(ErrTypes::InvalidType(arg2, Types::Float(0.0))),
            };
            let ud = match arg1 {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    arg1,
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            window.style.scale.0 = scale_x;
            return Ok(Types::Void);
        }
        // Window::scaleY
        66 => {
            let args = m.args();
            let arg1 = args[0];
            let arg2 = args[1];
            let scale_y = match arg2 {
                Types::Float(f) => f as f32,
                _ => return Err(ErrTypes::InvalidType(arg2, Types::Float(0.0))),
            };
            let ud = match arg1 {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    arg1,
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            window.style.scale.1 = scale_y;
            return Ok(Types::Void);
        }
        // Window::outlineColor
        67 => {
            let args = m.args();
            let arg1 = args[0];
            let arg2 = args[1];
            let color = match arg2 {
                Types::Pointer(pos, PointerTypes::UserData) => {
                    let color = m.user_data.data[pos].as_mut();
                    let color = color.as_any_mut().downcast_mut::<WinColor>().unwrap();
                    color.clone()
                }
                _ => Err(ErrTypes::InvalidType(
                    arg2,
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let ud = match arg1 {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    arg1,
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            window.style.outline_color = Color {
                r: color.r,
                g: color.g,
                b: color.b,
                a: color.a,
            };
            return Ok(Types::Void);
        }
        // Window::outlineThickness
        68 => {
            let args = m.args();
            let arg1 = args[0];
            let arg2 = args[1];
            let thickness = match arg2 {
                Types::Float(f) => f as f32,
                _ => return Err(ErrTypes::InvalidType(arg2, Types::Float(0.0))),
            };
            let ud = match arg1 {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    arg1,
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            window.style.outline_thickness = thickness;
            return Ok(Types::Void);
        }
        // Window::font
        69 => {
            let args = m.args();
            let arg1 = args[0];
            let arg2 = args[1];
            let font = match arg2 {
                Types::Pointer(pos, PointerTypes::UserData) => {
                    let font = m.user_data.data[pos].as_mut();
                    font.as_any_mut()
                        .downcast_mut::<WinFont>()
                        .unwrap()
                        .path
                        .clone()
                }
                _ => Err(ErrTypes::InvalidType(
                    arg2,
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let ud = match arg1 {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    arg1,
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            match Font::from_file(&font) {
                Some(font) => window.style.font = Some(Arc::new(font)),
                None => (),
            };
            return Ok(Types::Bool(window.style.font.is_some()));
        }
        // Window::fontSize
        70 => {
            let args = m.args();
            let arg1 = args[0];
            let arg2 = args[1];
            let size = match arg2 {
                Types::Uint(i) => i as u32,
                _ => return Err(ErrTypes::InvalidType(arg2, Types::Uint(0))),
            };
            let ud = match arg1 {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        arg1,
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            window.style.font_size = size;
            return Ok(Types::Void);
        }
        // Window::characterSpacing
        71 => {
            let args = m.args();
            let arg1 = args[0];
            let arg2 = args[1];
            let spacing = match arg2 {
                Types::Float(f) => f as f32,
                _ => return Err(ErrTypes::InvalidType(arg2, Types::Float(0.0))),
            };
            let ud = match arg1 {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        arg1,
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };

            let draw_style = m.user_data.data[ud].as_mut();
            let draw_style = draw_style.as_any_mut().downcast_mut::<DrawStyle>().unwrap();
            draw_style.character_spacing = spacing;
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // Window::lineSpacing
        72 => {
            let args = m.args();
            let arg1 = args[0];
            let arg2 = args[1];
            let spacing = match arg2 {
                Types::Float(f) => f as f32,
                _ => return Err(ErrTypes::InvalidType(arg2, Types::Float(0.0))),
            };
            let ud = match arg1 {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        arg1,
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };

            let draw_style = m.user_data.data[ud].as_mut();
            let draw_style = draw_style.as_any_mut().downcast_mut::<DrawStyle>().unwrap();
            draw_style.line_spacing = spacing;
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // Window::styledText
        73 => {
            let args = m.args();
            let arg1 = args[1];
            let arg2 = args[2];
            let arg3 = args[3];
            let arg4 = args[4];
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let x = match arg1 {
                Types::Float(f) => f as f32,
                _ => Err(ErrTypes::InvalidType(arg1, Types::Float(0.0)))?,
            };
            let y = match arg2 {
                Types::Float(f) => f as f32,
                _ => Err(ErrTypes::InvalidType(arg2, Types::Float(0.0)))?,
            };
            let text = match arg3 {
                Types::Pointer(pos, PointerTypes::String) => &m.strings.pool[pos],
                _ => Err(ErrTypes::InvalidType(
                    arg3,
                    Types::Pointer(0, PointerTypes::String),
                ))?,
            };
            let style = match arg4 {
                Types::Pointer(pos, PointerTypes::UserData) => {
                    let style = m.user_data.data[pos].as_mut();
                    style.as_any().downcast_ref::<DrawStyle>().unwrap()
                }
                _ => Err(ErrTypes::InvalidType(
                    arg4,
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let font = match &style.font {
                Some(font) => font,
                None => {
                    return Err(ErrTypes::Message(
                        "Window::drawText: no font set".to_string(),
                    ))
                }
            };
            let font = font.clone();
            let mut text = Text::new(text, &font, style.font_size);
            text.set_position((x, y));
            text.set_fill_color(style.color);
            text.set_outline_color(style.outline_color);
            text.set_outline_thickness(style.outline_thickness);
            text.set_rotation(style.rotation);
            text.set_scale((style.scale.0, style.scale.1));
            text.set_letter_spacing(style.character_spacing);
            text.set_line_spacing(style.line_spacing);
            text.set_origin(style.origin);
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            window.window.draw(&text);
            return Ok(Types::Void);
        }
        // Window::save
        74 => {
            let args = m.args();
            let arg1 = args[0];
            let ud = match arg1 {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        arg1,
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            let ds = window.style.clone();
            window.style_stack.push(ds);
            return Ok(Types::Void);
        }
        // Window::restore
        75 => {
            let args = m.args();
            let arg1 = args[0];
            let ud = match arg1 {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        arg1,
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            match window.style_stack.pop() {
                Some(style) => {
                    window.style = style;
                }
                None => (),
            }
            return Ok(Types::Void);
        }
        // Event::resizeX
        76 => {
            let args = m.args();
            let arg1 = args[0];
            let ud = match arg1 {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        arg1,
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };
            let event = m.user_data.data[ud].as_mut();
            let event = event.as_any_mut().downcast_mut::<Event>().unwrap();
            let size = match event.event {
                window::Event::Resized { width, .. } => width,
                _ => 0,
            };
            return Ok(Types::Uint(size as usize));
        }
        // Event::resizeY
        77 => {
            let args = m.args();
            let arg1 = args[0];
            let ud = match arg1 {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        arg1,
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };
            let event = m.user_data.data[ud].as_mut();
            let event = event.as_any_mut().downcast_mut::<Event>().unwrap();
            let size = match event.event {
                window::Event::Resized { height, .. } => height,
                _ => 0,
            };
            return Ok(Types::Uint(size as usize));
        }
        // DrawStyle::origin
        78 => {
            let args = m.args();
            let arg1 = args[0];
            let arg2 = args[1];
            let arg3 = args[2];
            let x = match arg2 {
                Types::Float(f) => f as f32,
                _ => return Err(ErrTypes::InvalidType(arg2, Types::Float(0.0))),
            };
            let y = match arg3 {
                Types::Float(f) => f as f32,
                _ => return Err(ErrTypes::InvalidType(arg3, Types::Float(0.0))),
            };
            let ud = match arg1 {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        arg1,
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };

            let draw_style = m.user_data.data[ud].as_mut();
            let draw_style = draw_style.as_any_mut().downcast_mut::<DrawStyle>().unwrap();
            draw_style.origin = (x, y);
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // alert
        79 => {
            let args = m.args();
            let arg1 = args[0];
            let arg2 = args[1];
            let title = match arg1 {
                Types::Pointer(pos, PointerTypes::String) => &m.strings.pool[pos],
                _ => Err(ErrTypes::InvalidType(
                    arg1,
                    Types::Pointer(0, PointerTypes::String),
                ))?,
            };
            let message = match arg2 {
                Types::Pointer(pos, PointerTypes::String) => &m.strings.pool[pos],
                _ => Err(ErrTypes::InvalidType(
                    arg2,
                    Types::Pointer(0, PointerTypes::String),
                ))?,
            };
            let mut alert = Alert::new(AlertType::Alert, title, message);
            match alert.show() {
                AlertResponse::Alert(ok) => return Ok(Types::Bool(ok)),
                _ => return Ok(Types::Bool(false)),
            }
        }
        // prompt
        80 => {
            let args = m.args();
            let arg1 = args[0];
            let arg2 = args[1];
            let arg3 = args[2];
            let title = match arg1 {
                Types::Pointer(pos, PointerTypes::String) => &m.strings.pool[pos],
                _ => Err(ErrTypes::InvalidType(
                    arg1,
                    Types::Pointer(0, PointerTypes::String),
                ))?,
            };
            let message = match arg2 {
                Types::Pointer(pos, PointerTypes::String) => &m.strings.pool[pos],
                _ => Err(ErrTypes::InvalidType(
                    arg2,
                    Types::Pointer(0, PointerTypes::String),
                ))?,
            };
            let default = match arg3 {
                Types::Pointer(pos, PointerTypes::String) => &m.strings.pool[pos],
                _ => Err(ErrTypes::InvalidType(
                    arg3,
                    Types::Pointer(0, PointerTypes::String),
                ))?,
            };
            let mut prompt = Alert::new(AlertType::Prompt(default.clone()), title, message);
            match prompt.show() {
                AlertResponse::Prompt(res) => {
                    let str = ctx.memory.strings.from_string(res);
                    return Ok(Types::Pointer(str, PointerTypes::String));
                }
                _ => return Ok(Types::Null),
            }
        }
        // confirm
        81 => {
            let args = m.args();
            let arg1 = args[0];
            let arg2 = args[1];
            let title = match arg1 {
                Types::Pointer(pos, PointerTypes::String) => &m.strings.pool[pos],
                _ => Err(ErrTypes::InvalidType(
                    arg1,
                    Types::Pointer(0, PointerTypes::String),
                ))?,
            };
            let message = match arg2 {
                Types::Pointer(pos, PointerTypes::String) => &m.strings.pool[pos],
                _ => Err(ErrTypes::InvalidType(
                    arg2,
                    Types::Pointer(0, PointerTypes::String),
                ))?,
            };
            let mut confirm = Alert::new(AlertType::Confirm, title, message);
            match confirm.show() {
                AlertResponse::Confirm(ok) => return Ok(Types::Bool(ok)),
                _ => return Ok(Types::Bool(false)),
            }
        }
        // Image::new
        82 => {
            let args = m.args();
            let arg1 = args[0];
            let path = match arg1 {
                Types::Pointer(pos, PointerTypes::String) => &m.strings.pool[pos],
                _ => Err(ErrTypes::InvalidType(
                    arg1,
                    Types::Pointer(0, PointerTypes::String),
                ))?,
            };
            match WinImage::new(path) {
                Some(image) => {
                    let ud = ctx.memory.user_data.push(Box::new(image));
                    return Ok(Types::Pointer(ud, PointerTypes::UserData));
                }
                None => return Err(ErrTypes::Message("Image::new: failed".to_string())),
            }
        }
        // Image::width
        83 => {
            let args = m.args();
            let arg1 = args[0];
            let ud = match arg1 {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    arg1,
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let image = m.user_data.data[ud].as_mut();
            let image = image.as_any_mut().downcast_mut::<WinImage>().unwrap();
            return Ok(Types::Uint(image.texture.size().x as usize));
        }
        // Image::height
        84 => {
            let args = m.args();
            let arg1 = args[0];
            let ud = match arg1 {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    arg1,
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let image = m.user_data.data[ud].as_mut();
            let image = image.as_any_mut().downcast_mut::<WinImage>().unwrap();
            return Ok(Types::Uint(image.texture.size().y as usize));
        }
        // Window::drawImage
        85 => {
            let args = m.args();
            let win = args[0];
            let x = args[1];
            let y = args[2];
            let image = args[3];
            let win_ud = match win {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(win, Types::Pointer(0, PointerTypes::UserData)))?,
            };
            let image_ud = match image {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    image,
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let x = match x {
                Types::Float(f) => f as f32,
                _ => Err(ErrTypes::InvalidType(x, Types::Float(0.0)))?,
            };
            let y = match y {
                Types::Float(f) => f as f32,
                _ => Err(ErrTypes::InvalidType(y, Types::Float(0.0)))?,
            };
            
            let image = m.user_data.data[image_ud].as_mut();
            let image = image.as_any_mut().downcast_mut::<WinImage>().unwrap().texture.to_owned();
            let window = m.user_data.data[win_ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            let mut sprite = Sprite::with_texture(&image);
            sprite.set_position((x, y));
            window.window.draw(&sprite);
        }
        // Window::drawLine
        86 => {
            let args = m.args();
            let win = args[0];
            let x1 = args[1];
            let y1 = args[2];
            let x2 = args[3];
            let y2 = args[4];
            let win_ud = match win {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(win, Types::Pointer(0, PointerTypes::UserData)))?,
            };
            let x1 = match x1 {
                Types::Float(f) => f as f32,
                _ => Err(ErrTypes::InvalidType(x1, Types::Float(0.0)))?,
            };
            let y1 = match y1 {
                Types::Float(f) => f as f32,
                _ => Err(ErrTypes::InvalidType(y1, Types::Float(0.0)))?,
            };
            let x2 = match x2 {
                Types::Float(f) => f as f32,
                _ => Err(ErrTypes::InvalidType(x2, Types::Float(0.0)))?,
            };
            let y2 = match y2 {
                Types::Float(f) => f as f32,
                _ => Err(ErrTypes::InvalidType(y2, Types::Float(0.0)))?,
            };
            let window = m.user_data.data[win_ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            use sfml::graphics::{VertexBuffer, Vertex, PrimitiveType};
            let mut line = VertexBuffer::new(PrimitiveType::LINES, 2, VertexBufferUsage::STREAM);
            let mut a = Vertex::with_pos(Vector2f::new(x1, y1));
            a.color = window.style.color;
            let mut b = Vertex::with_pos(Vector2f::new(x2, y2));
            b.color = window.style.color;
            line.update(&[a, b], 0);
            window.window.draw(&line);
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
        fun width(self=reg.ptr, width=reg.g1: uint?): uint > 8i
        fun height(self=reg.ptr, height=reg.g1: uint?): uint > 9i
        fun fps(self=reg.ptr, fps=reg.g1: uint) > 13i
        fun title(self=reg.ptr, title=reg.g1: string)> 15i

        fun clear(self=reg.ptr) > 5i
        fun display(self=reg.ptr) > 6i
        fun close(self=reg.ptr) > 7i

        fun poll(self=reg.ptr): Event? > 10i

        fun drawRectangle(
            self=reg.ptr,
            x=reg.g1: float,
            y=reg.g2: float,
            width=reg.g3: float,
            height=reg.g4: float,
        ) > 47i
        fun drawCircle(
            self=reg.ptr, 
            x=reg.g1: float, 
            y=reg.g2: float, 
            radius=reg.g3: float,
        ) > 48i
        fun drawText(
            self=reg.ptr, 
            x=reg.g1: float, 
            y=reg.g2: float, 
            text=reg.g3: string, 
        ) > 49i
        fun styledRectangle(
            self=reg.ptr,
            x=reg.g1: float,
            y=reg.g2: float,
            width=reg.g3: float,
            height=reg.g4: float,
            style=reg.g5: DrawStyle,
        ) > 50i
        fun styledCircle(
            self=reg.ptr, 
            x=reg.g1: float, 
            y=reg.g2: float, 
            radius=reg.g3: float, 
            style=reg.g4: DrawStyle,
        ) > 51i
        fun styledText(
            self=reg.ptr, 
            x=reg.g1: float, 
            y=reg.g2: float, 
            text=reg.g3: string, 
            style=reg.g4: DrawStyle,
        ) > 73i
        fun drawImage(
            self=reg.ptr, 
            x=reg.g1: float, 
            y=reg.g2: float, 
            image=reg.g3: Image,
        ) > 85i
        fun drawLine(
            self=reg.ptr, 
            x1=reg.g1: float, 
            y1=reg.g2: float, 
            x2=reg.g3: float, 
            y2=reg.g4: float,
        ) > 86i

        fun background(self=reg.ptr, color=reg.g1: Color) > 60i
        fun getStyle(self=reg.ptr): DrawStyle > 61i
        fun setStyle(self=reg.ptr, style=reg.g1: DrawStyle) > 62i
        fun color(self=reg.ptr, color=reg.g1: Color) > 63i
        fun rotation(self=reg.ptr, rotation=reg.g1: float) > 64i
        fun scaleX(self=reg.ptr, scaleX=reg.g1: float) > 65i
        fun scaleY(self=reg.ptr, scaleY=reg.g1: float) > 66i
        fun outlineColor(self=reg.ptr, outlineColor=reg.g1: Color) > 67i
        fun outlineThickness(self=reg.ptr, outlineThickness=reg.g1: float) > 68i
        fun font(self=reg.ptr, font=reg.g1: Font): bool > 69i
        fun fontSize(self=reg.ptr, fontSize=reg.g1: uint) > 70i
        fun characterSpacing(self=reg.ptr, characterSpacing=reg.g1: float) > 71i
        fun lineSpacing(self=reg.ptr, lineSpacing=reg.g1: float) > 72i

        fun save(self=reg.ptr) > 74i
        fun restore(self=reg.ptr) > 75i
    }

    userdata WinBuilder > 1i {
        new () > 1i
        fun build(self=reg.ptr): Window > 54i

        fun title(self=reg.ptr, title=reg.g1: string): WinBuilder > 2i
        fun width(self=reg.ptr, width=reg.g1: uint): WinBuilder > 3i
        fun height(self=reg.ptr, height=reg.g1: uint): WinBuilder > 4i
        fun resize(self=reg.ptr): WinBuilder > 55i
        fun fullscreen(self=reg.ptr): WinBuilder > 56i
        fun close(self=reg.ptr): WinBuilder > 57i
        fun titlebar(self=reg.ptr): WinBuilder > 58i
        fun default(self=reg.ptr): WinBuilder > 59i
    }

    fun alert(title=reg.g1: string, message=reg.g2: string): bool > 79i
    fun prompt(title=reg.g1: string, message=reg.g2: string, default=reg.g3: string): string > 80i
    fun confirm(title=reg.g1: string, message=reg.g2: string): bool > 81i

    userdata Event > 2i {
        fun code(self=reg.ptr): Events > 11i

        fun key(self=reg.ptr): Keys > 12i
        fun scan(self=reg.ptr): Scan > 16i
        fun input(self=reg.ptr): char > 14i
        fun alt(self=reg.ptr): bool > 22i
        fun control(self=reg.ptr): bool > 23i
        fun shift(self=reg.ptr): bool > 24i
        fun system(self=reg.ptr): bool > 25i

        fun verticalWheel(self=reg.ptr): float > 17i
        fun horizontalWheel(self=reg.ptr): float > 18i
        fun mouseX(self=reg.ptr): uint > 19i
        fun mouseY(self=reg.ptr): uint > 20i
        fun mouseButton(self=reg.ptr): MouseButton > 21i

        fun resizeX(self=reg.ptr): uint > 76i
        fun resizeY(self=reg.ptr): uint > 77i
    }

    userdata DrawStyle > 3i {
        new () > 26i

        fun color(self=reg.ptr, color=reg.g1: Color): DrawStyle > 27i
        fun rotation(self=reg.ptr, rotation=reg.g1: float): DrawStyle > 35i
        fun scaleX(self=reg.ptr, scaleX=reg.g1: float): DrawStyle > 36i
        fun scaleY(self=reg.ptr, scaleY=reg.g1: float): DrawStyle > 37i
        fun outlineColor(self=reg.ptr, outlineColor=reg.g1: Color): DrawStyle > 38i
        fun outlineThickness(self=reg.ptr, outlineThickness=reg.g1: float): DrawStyle > 39i
        fun font(self=reg.ptr, font=reg.g1: Font): DrawStyle > 40i
        fun fontSize(self=reg.ptr, fontSize=reg.g1: uint): DrawStyle > 41i
        fun characterSpacing(self=reg.ptr, characterSpacing=reg.g1: float): DrawStyle > 52i
        fun lineSpacing(self=reg.ptr, lineSpacing=reg.g1: float): DrawStyle > 53i
        fun origin(self=reg.ptr, x=reg.g1: float, y=reg.g2: float): DrawStyle > 78i
    }

    userdata Color > 4i {
        new (r=reg.g1: uint, g=reg.g2: uint, b=reg.g3: uint, a=reg.g4: uint) > 28i

        fun r(self=reg.ptr, set=reg.g1: uint?): uint > 29i
        fun g(self=reg.ptr, set=reg.g1: uint?): uint > 30i
        fun b(self=reg.ptr, set=reg.g1: uint?): uint > 31i
        fun a(self=reg.ptr, set=reg.g1: uint?): uint > 32i

        fun FromHex(hex=reg.g1: string): Color > 33i
        fun Copy(self=reg.ptr): Color > 34i
        fun From(color=reg.g1: Colors): Color > 46i
    }

    userdata Image > 5i {
        new (path=reg.g1: string) > 82i

        fun width(self=reg.ptr): uint > 83i
        fun height(self=reg.ptr): uint > 84i
    }

    userdata Font > 5i {
        new (path=reg.g1: string) > 42i

        fun default (name=reg.g1: string): Font > 43i
        fun ubuntuMono (): Font > 44i
        fun roboto (): Font > 45i
    }

    enum Events > 0i {
        Closed
        Resized
        LostFocus
        GainedFocus
        Input
        KeyPressed
        KeyReleased
        Wheel
        MousePressed
        MouseReleased
        MouseMoved

        Unknown
        None
    }

    enum Keys > 1i { 
        A B C D E F G H I J K L M N O P Q R S T U V W X Y Z 
        Num0 Num1 Num2 Num3 Num4 Num5 Num6 Num7 Num8 Num9 
        Escape LControl LShift LAlt LSystem RControl RShift RAlt RSystem Menu LBracket RBracket 
        Semicolon Comma Period Quote Slash Backslash Tilde Equal Hyphen Space 
        Enter Backspace Tab PageUp PageDown End Home Insert Delete 
        Add Subtract Multiply Divide Left Right Up Down 
        Numpad0 Numpad1 Numpad2 Numpad3 Numpad4 Numpad5 Numpad6 Numpad7 Numpad8 Numpad9 
        F1 F2 F3 F4 F5 F6 F7 F8 F9 F10 F11 F12 F13 F14 F15 
        Pause
    }
    enum Scan > 2i { 
        A B C D E F G H I J K L M N O P Q R S T U V W X Y Z 
        Num1 Num2 Num3 Num4 Num5 Num6 Num7 Num8 Num9 Num0 
        Enter Escape Backspace Tab Space Hyphen Equal LBracket RBracke Backslash 
        Semicolon Apostrophe Grave Comma Period Slash 
        F1 F2 F3 F4 F5 F6 F7 F8 F9 F10 F11 F12 F13 F14 F15 F16 F17 F18 F19 F20 F21 F22 F23 F24 
        CapsLock PrintScreen ScrollLock Pause Insert Home PageUp Delete End PageDown 
        Right Left Down Up 
        NumLock NumpadDivide NumpadMultiply NumpadMinus NumpadPlus NumpadEqual NumpadEnter NumpadDecimal 
        Numpad1 Numpad2 Numpad3 Numpad4 Numpad5 Numpad6 Numpad7 Numpad8 Numpad9 Numpad 
        NonUsBackslash Application Execute ModeChange Help Menu 
        Select Redo Undo Cut Copy Paste 
        VolumeMute VolumeUp VolumeDown MediaPlayPause 
        MediaStop MediaNextTrack MediaPreviousTrack 
        LControl LShift LAlt LSystem RControl RShift RAlt RSystem 
        Back Forward Refresh Stop Search Favorites HomePage 
        LaunchApplication1 LaunchApplication2 LaunchMail LaunchMediaSelect 
        ScancodeCount 
    }
    enum MouseButton > 3i {
        Left Right Middle 
        XButton1 XButton2
    }
    enum Colors > 4i {
        Black White Red Green Blue Yellow Magenta Cyan Transparent
    }
    "#.to_string()
}

#[no_mangle]
pub fn init(
    _ctx: &mut Context,
    my_id: usize,
) -> fn(&mut Context, usize, usize) -> Result<Types, runtime_error::ErrTypes> {
    call
}

struct Window {
    window: RenderWindow,
    /// Background color for clearing the window
    bg: Color,
    /// Style for drawing
    style: DrawStyle,
    /// Stack for saving and restoring the style
    style_stack: Vec<DrawStyle>,
    lib_id: usize,
}
impl Window {
    fn new(size: (u32, u32), title: &str, style: window::Style, settings: &WinBuilder) -> Self {
        let window = RenderWindow::new(size, title, style, &Default::default());
        let mut style = DrawStyle::new();
        match unsafe { Font::from_memory(include_bytes!("../../sfml/fonts/Roboto-Regular.ttf")) } {
            Some(font) => style.font = Some(Arc::new(font)),
            None => (),
        }
        Self {
            window,
            bg: Color::BLACK,
            style,
            style_stack: Vec::new(),
            lib_id: 0,
        }
    }
}
impl UserData for Window {
    fn label(&self) -> &str {
        &"Window"
    }

    fn id(&self) -> usize {
        0
    }

    fn lib_id(&self) -> usize {
        self.lib_id
    }

    fn gc_method(&self) -> &user_data::GcMethod {
        &user_data::GcMethod::Gc
    }

    fn cleanup(&mut self) {}
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[derive(Clone)]
struct DrawStyle {
    color: Color,
    rotation: f32,
    scale: (f32, f32),
    origin: (f32, f32),
    outline_color: Color,
    outline_thickness: f32,
    font: Option<Arc<SfBox<Font>>>,
    font_size: u32,
    character_spacing: f32,
    line_spacing: f32,
    lib_id: usize,
}
impl DrawStyle {
    /// Creates a new DrawStyle
    fn new() -> Self {
        Self {
            color: Color::WHITE,
            rotation: 0.0,
            scale: (1.0, 1.0),
            origin: (0.0, 0.0),
            outline_color: Color::TRANSPARENT,
            outline_thickness: 0.0,
            font: None,
            font_size: 30,
            character_spacing: 1.0,
            line_spacing: 1.0,
            lib_id: 0,
        }
    }
}
impl UserData for DrawStyle {
    fn label(&self) -> &str {
        &"DrawStyle"
    }

    fn id(&self) -> usize {
        0
    }

    fn lib_id(&self) -> usize {
        self.lib_id
    }

    fn gc_method(&self) -> &user_data::GcMethod {
        &user_data::GcMethod::Gc
    }

    fn cleanup(&mut self) {}
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
    style: window::Style,
    lib_id: usize,
}
impl WinBuilder {
    fn new() -> Self {
        Self {
            title: "Window".to_string(),
            width: 800,
            height: 600,
            style: window::Style::NONE,
            lib_id: 0,
        }
    }
}
impl UserData for WinBuilder {
    fn label(&self) -> &str {
        &"WinBuilder"
    }

    fn id(&self) -> usize {
        1
    }

    fn lib_id(&self) -> usize {
        self.lib_id
    }

    fn gc_method(&self) -> &user_data::GcMethod {
        &user_data::GcMethod::Gc
    }

    fn cleanup(&mut self) {}
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

struct Event {
    lib_id: usize,
    event: window::Event,
}
impl Event {
    fn new(event: window::Event) -> Self {
        Self { lib_id: 0, event }
    }
}
impl UserData for Event {
    fn label(&self) -> &str {
        &"Event"
    }

    fn id(&self) -> usize {
        2
    }

    fn lib_id(&self) -> usize {
        self.lib_id
    }

    fn gc_method(&self) -> &user_data::GcMethod {
        &user_data::GcMethod::Gc
    }

    fn cleanup(&mut self) {}
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[derive(Clone)]
struct WinColor {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
    lib_id: usize,
}
impl WinColor {
    fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r,
            g,
            b,
            a,
            lib_id: 0,
        }
    }
    fn from_hex(hex: &str) -> Self {
        let hex = hex.trim_start_matches('#');
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap();
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap();
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap();
        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16).unwrap()
        } else {
            255
        };
        Self {
            r,
            g,
            b,
            a,
            lib_id: 0,
        }
    }
}
impl UserData for WinColor {
    fn label(&self) -> &str {
        &"Color"
    }

    fn id(&self) -> usize {
        3
    }

    fn lib_id(&self) -> usize {
        self.lib_id
    }

    fn gc_method(&self) -> &user_data::GcMethod {
        &user_data::GcMethod::Gc
    }

    fn cleanup(&mut self) {}
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

struct WinFont {
    path: String,
    lib_id: usize,
}
impl WinFont {
    fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            lib_id: 0,
        }
    }
    fn default(name: &str) -> Self {
        let ruda_path = std::env::var("RUDA_PATH").unwrap_or_else(|_| ".".to_string());
        let path = format!("{}/fonts/{}.ttf", ruda_path, name);
        Self { path, lib_id: 0 }
    }
    fn ubuntu_mono() -> Self {
        let ruda_path = std::env::var("RUDA_PATH").unwrap_or_else(|_| ".".to_string());
        let path = format!("{}/fonts/UbuntuMono-Regular.ttf", ruda_path);
        Self { path, lib_id: 0 }
    }
    fn roboto() -> Self {
        let ruda_path = std::env::var("RUDA_PATH").unwrap_or_else(|_| ".".to_string());
        let path = format!("{}/fonts/Roboto-Regular.ttf", ruda_path);
        Self { path, lib_id: 0 }
    }
}
impl UserData for WinFont {
    fn label(&self) -> &str {
        &"Font"
    }

    fn id(&self) -> usize {
        4
    }

    fn lib_id(&self) -> usize {
        self.lib_id
    }

    fn gc_method(&self) -> &user_data::GcMethod {
        &user_data::GcMethod::Gc
    }

    fn cleanup(&mut self) {}
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[derive(Clone)]
struct WinImage {
    texture: Arc<SfBox<sfml::graphics::Texture>>,
    lib_id: usize,
}

impl WinImage {
    fn new(path: &str) -> Option<Self> {
        let texture = match sfml::graphics::Texture::from_file(path) {
            sfml::LoadResult::Ok(texture) => Arc::new(texture),
            sfml::LoadResult::Err(_) => return None,
        };
        Some(Self {
                    texture,
                    lib_id: 0,
                })
    }
}

impl UserData for WinImage {
    fn label(&self) -> &str {
        &"Image"
    }

    fn id(&self) -> usize {
        5
    }

    fn lib_id(&self) -> usize {
        self.lib_id
    }

    fn gc_method(&self) -> &user_data::GcMethod {
        &user_data::GcMethod::Gc
    }

    fn cleanup(&mut self) {}
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Blocking alert
struct Alert {
    alert_type: AlertType,
    title: String,
    message: String,
}

impl Alert {
    fn new(alert_type: AlertType, title: &str, message: &str) -> Self {
        Self {
            alert_type,
            title: title.to_string(),
            message: message.to_string(),
        }
    }

    fn show(&mut self) -> AlertResponse {
        // sfml window
        let window = &mut RenderWindow::new(
            (400, 200),
            &self.title,
            window::Style::CLOSE,
            &Default::default(),
        );
        window.set_vertical_sync_enabled(true);
        // draw the window
        let font = match unsafe {
            Font::from_memory(include_bytes!("../../sfml/fonts/Roboto-Regular.ttf"))
        } {
            Some(font) => font,
            None => {
                println!("Failed to load font");
                return AlertResponse::FontNotFound;
            }
        };
        let mut text = Text::new(&self.message, &font, 20);
        text.set_position((10.0, 10.0));
        text.set_fill_color(Color::BLACK);
        let mut input_field = match &self.alert_type {
            AlertType::Prompt(txt) => Some(ImTextField::new(&txt, (10.0, 100.0), (380.0, 40.0))),
            _ => None,
        };
        let mut focused = true;
        while window.is_open() {
            let mut mouse_pressed = false;
            let mut character = None;
            while let Some(event) = window.poll_event() {
                match event {
                    window::Event::Closed => {
                        window.close();
                        match self.alert_type {
                            AlertType::Alert => return AlertResponse::Alert(false),
                            AlertType::Prompt(_) => return AlertResponse::Prompt(String::new()),
                            AlertType::Confirm => return AlertResponse::Confirm(false),
                        }
                    }
                    window::Event::KeyPressed { code, .. } => match code {
                        window::Key::Escape => {
                            window.close();
                            match self.alert_type {
                                AlertType::Alert => return AlertResponse::Alert(false),
                                AlertType::Prompt(_) => return AlertResponse::Prompt(String::new()),
                                AlertType::Confirm => return AlertResponse::Confirm(false),
                            }
                        }
                        window::Key::Enter => {
                            window.close();
                            match self.alert_type {
                                AlertType::Alert => return AlertResponse::Alert(true),
                                AlertType::Prompt(_) => {
                                    return AlertResponse::Prompt(input_field.unwrap().text)
                                }
                                AlertType::Confirm => return AlertResponse::Confirm(true),
                            }
                        }
                        window::Key::Backspace => {
                            if let Some(input_field) = input_field.as_mut() {
                                input_field.text.pop();
                            }
                        }

                        _ => (),
                    },
                    window::Event::TextEntered { unicode, .. } => {
                        // check if the character is valid
                        if unicode.is_ascii_control() {
                            continue;
                        }
                        character = Some(unicode as char);
                    }
                    window::Event::LostFocus => focused = false,
                    window::Event::GainedFocus => focused = true,
                    window::Event::MouseButtonPressed { button, .. } => {
                        if button == window::mouse::Button::Left {
                            mouse_pressed = true;
                        }
                    }
                    _ => (),
                }
            }
            window.clear(Color::WHITE);
            window.draw(&text);
            match self.alert_type {
                AlertType::Alert => {
                    if self.imbutton(window, "OK", (150.0, 150.0), &font, mouse_pressed) {
                        window.close();
                        return AlertResponse::Alert(true);
                    }
                }
                AlertType::Prompt(_) => {
                    if self.imbutton(window, "OK", (80.0, 150.0), &font, mouse_pressed) {
                        window.close();
                        return AlertResponse::Prompt(input_field.unwrap().text);
                    }
                    if self.imbutton(window, "Cancel", (220.0, 150.0), &font, mouse_pressed) {
                        window.close();
                        return AlertResponse::Prompt(String::new());
                    }
                    input_field
                        .as_mut()
                        .unwrap()
                        .draw(window, &font, mouse_pressed, character);
                }
                AlertType::Confirm => {
                    if self.imbutton(window, "OK", (80.0, 150.0), &font, mouse_pressed) {
                        window.close();
                        return AlertResponse::Confirm(true);
                    }
                    if self.imbutton(window, "Cancel", (220.0, 150.0), &font, mouse_pressed) {
                        window.close();
                        return AlertResponse::Confirm(false);
                    }
                }
                _ => todo!(),
            }
            window.display();
        }
        AlertResponse::Err
    }

    fn imbutton(
        &mut self,
        window: &mut RenderWindow,
        text: &str,
        pos: (f32, f32),
        font: &Font,
        mouse_pressed: bool,
    ) -> bool {
        let mut button = RectangleShape::new();
        button.set_size((100.0, 45.0));
        button.set_position(pos);
        button.set_fill_color(Color::BLACK);
        window.draw(&button);
        let mut text = Text::new(text, font, 20);
        text.set_position((pos.0 + 10.0, pos.1 + 10.0));
        text.set_fill_color(Color::WHITE);
        window.draw(&text);
        let mouse_pos = window.mouse_position();
        let mouse_pos = (mouse_pos.x as f32, mouse_pos.y as f32);
        if mouse_pos.0 > pos.0
            && mouse_pos.0 < pos.0 + 100.0
            && mouse_pos.1 > pos.1
            && mouse_pos.1 < pos.1 + 50.0
        {
            if mouse_pressed {
                return true;
            }
        }
        return false;
    }
}

enum AlertType {
    Prompt(String),
    Confirm,
    Alert,
}

enum AlertResponse {
    Prompt(String),
    Confirm(bool),
    Alert(bool),
    FontNotFound,
    Err,
}

struct ImTextField {
    text: String,
    selected: bool,
    ghost_text: String,
    pos: (f32, f32),
    size: (f32, f32),
    creation_time: std::time::Instant,
}

impl ImTextField {
    fn new(text: &str, pos: (f32, f32), size: (f32, f32)) -> Self {
        Self {
            text: text.to_string(),
            selected: true,
            pos,
            size,
            ghost_text: text.to_string(),
            creation_time: std::time::Instant::now(),
        }
    }

    fn draw(
        &mut self,
        window: &mut RenderWindow,
        font: &Font,
        mouse_pressed: bool,
        character: Option<char>,
    ) -> bool {
        let mut button = RectangleShape::new();
        button.set_size(self.size);
        button.set_position(self.pos);
        button.set_fill_color(Color::BLACK);
        window.draw(&button);
        let mut text = Text::new(&self.text, font, 20);
        text.set_position((self.pos.0 + 10.0, self.pos.1 + 10.0));
        text.set_fill_color(Color::WHITE);
        window.draw(&text);
        // ghost text
        if self.text.is_empty() {
            let mut text = Text::new(&self.ghost_text, font, 20);
            text.set_position((self.pos.0 + 10.0, self.pos.1 + 10.0));
            text.set_fill_color(Color {
                r: 255,
                g: 255,
                b: 255,
                a: 100,
            });
            window.draw(&text);
        }
        // flashing cursor based on time
        let mut cursor = RectangleShape::new();
        cursor.set_size((2.0, 20.0));
        cursor.set_position((
            self.pos.0 + 10.0 + text.local_bounds().width + 2.0,
            self.pos.1 + 10.0,
        ));
        cursor.set_fill_color(Color::WHITE);
        if self.selected {
            if self.creation_time.elapsed().as_millis() % 1000 < 500 {
                window.draw(&cursor);
            }
        }
        let mouse_pos = window.mouse_position();
        let mouse_pos = (mouse_pos.x as f32, mouse_pos.y as f32);
        if mouse_pos.0 > self.pos.0
            && mouse_pos.0 < self.pos.0 + self.size.0
            && mouse_pos.1 > self.pos.1
            && mouse_pos.1 < self.pos.1 + self.size.1
        {
            if mouse_pressed {
                self.selected = true;
                // reset the creation time
                self.creation_time = std::time::Instant::now();
            }
        } else {
            if mouse_pressed {
                self.selected = false;
            }
        }
        if self.selected {
            if let Some(character) = character {
                self.text.push(character);
                // reset the creation time
                self.creation_time = std::time::Instant::now();
            }
        }
        return false;
    }
}
