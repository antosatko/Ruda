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
use sfml::graphics::{
    CircleShape, Color, Font, Rect, RectangleShape, RenderTarget, RenderWindow, Shape, Text,
    TextStyle, Transformable, Image,
};
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
                    unsafe { window.window.set_icon(icon.size().x, icon.size().y, icon.pixel_data()) };
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
            window.window.clear(Color::BLACK);
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
            return Ok(Types::Void);
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
            return Ok(Types::Void);
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
            return Ok(Types::Void);
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
            return Ok(Types::Void);
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
            return Ok(Types::Void);
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
            return Ok(Types::Void);
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
            draw_style.font = Font::from_file(&font);
            return Ok(Types::Void);
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
            return Ok(Types::Void);
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
            let color = match arg5 {
                Types::Pointer(pos, PointerTypes::UserData) => {
                    let color = m.user_data.data[pos].as_mut();
                    let color = color.as_any_mut().downcast_mut::<WinColor>().unwrap();
                    color.clone()
                }
                _ => Err(ErrTypes::InvalidType(
                    arg5,
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            let mut rect = RectangleShape::new();
            rect.set_position((x, y));
            rect.set_size((width, height));
            rect.set_fill_color(Color {
                r: color.r,
                g: color.g,
                b: color.b,
                a: color.a,
            });
            window.window.draw(&rect);
            return Ok(Types::Void);
        }
        // Window::drawCircle
        48 => {
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
            let color = match arg4 {
                Types::Pointer(pos, PointerTypes::UserData) => {
                    let color = m.user_data.data[pos].as_mut();
                    let color = color.as_any_mut().downcast_mut::<WinColor>().unwrap();
                    color.clone()
                }
                _ => Err(ErrTypes::InvalidType(
                    arg4,
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            let mut circle = CircleShape::new(radius, 30);
            circle.set_position((x, y));
            circle.set_fill_color(Color {
                r: color.r,
                g: color.g,
                b: color.b,
                a: color.a,
            });
            window.window.draw(&circle);
            return Ok(Types::Void);
        }
        // Window::drawText
        49 => {
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
                    style.as_any_mut().downcast_mut::<DrawStyle>().unwrap()
                }
                _ => Err(ErrTypes::InvalidType(
                    arg4,
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let font = match style.font {
                Some(ref font) => font.clone(),
                None => {
                    return Err(ErrTypes::InvalidType(
                        Types::Null,
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };
            let mut text = Text::new(text, &font, style.font_size);
            text.set_position((x, y));
            text.set_fill_color(Color {
                r: style.color.r,
                g: style.color.g,
                b: style.color.b,
                a: style.color.a,
            });
            text.set_outline_color(Color {
                r: style.outline_color.r,
                g: style.outline_color.g,
                b: style.outline_color.b,
                a: style.outline_color.a,
            });
            text.set_outline_thickness(style.outline_thickness);
            text.set_rotation(style.rotation);
            text.set_scale((style.scale.0, style.scale.1));
            text.set_letter_spacing(style.character_spacing);
            text.set_line_spacing(style.line_spacing);
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
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
            rect.set_fill_color(Color {
                r: style.color.r,
                g: style.color.g,
                b: style.color.b,
                a: style.color.a,
            });
            rect.set_outline_color(Color {
                r: style.outline_color.r,
                g: style.outline_color.g,
                b: style.outline_color.b,
                a: style.outline_color.a,
            });
            rect.set_outline_thickness(style.outline_thickness);
            rect.set_rotation(style.rotation);
            rect.set_scale((style.scale.0, style.scale.1));
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
            circle.set_fill_color(Color {
                r: style.color.r,
                g: style.color.g,
                b: style.color.b,
                a: style.color.a,
            });
            circle.set_outline_color(Color {
                r: style.outline_color.r,
                g: style.outline_color.g,
                b: style.outline_color.b,
                a: style.outline_color.a,
            });
            circle.set_outline_thickness(style.outline_thickness);
            circle.set_rotation(style.rotation);
            circle.set_scale((style.scale.0, style.scale.1));
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
            return Ok(Types::Void);
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
            return Ok(Types::Void);
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
                        window.window.set_icon(icon.size().x, icon.size().y, icon.pixel_data())
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
            color=reg.g5: Color,
        ) > 47i
        fun drawCircle(
            self=reg.ptr, 
            x=reg.g1: float, 
            y=reg.g2: float, 
            radius=reg.g3: float, 
            color=reg.g4: Color,
        ) > 48i
        fun drawText(
            self=reg.ptr, 
            x=reg.g1: float, 
            y=reg.g2: float, 
            text=reg.g3: string, 
            style=reg.g5: DrawStyle,
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
    }

    userdata DrawStyle > 3i {
        new () > 26i

        fun color(self=reg.ptr, color=reg.g1: Color) > 27i
        fun rotation(self=reg.ptr, rotation=reg.g1: float) > 35i
        fun scaleX(self=reg.ptr, scaleX=reg.g1: float) > 36i
        fun scaleY(self=reg.ptr, scaleY=reg.g1: float) > 37i
        fun outlineColor(self=reg.ptr, outlineColor=reg.g1: Color) > 38i
        fun outlineThickness(self=reg.ptr, outlineThickness=reg.g1: float) > 39i
        fun font(self=reg.ptr, font=reg.g1: Font) > 40i
        fun fontSize(self=reg.ptr, fontSize=reg.g1: uint) > 41i
        fun characterSpacing(self=reg.ptr, characterSpacing=reg.g1: float) > 52i
        fun lineSpacing(self=reg.ptr, lineSpacing=reg.g1: float) > 53i
    }

    userdata Color > 4i {
        new (r=reg.g1: uint, g=reg.g2: uint, b=reg.g3: uint, a=reg.g4: uint) > 28i

        fun r(self=reg.ptr, set=reg.g1: uint?): uint > 29i
        fun g(self=reg.ptr, set=reg.g1: uint?): uint > 30i
        fun b(self=reg.ptr, set=reg.g1: uint?): uint > 31i
        fun a(self=reg.ptr, set=reg.g1: uint?): uint > 32i

    }
    fun ColorFromHex(hex=reg.g1: string): Color > 33i
    fun ColorCopy(self=reg.ptr): Color > 34i
    fun ColorFrom(color=reg.g1: Colors): Color > 46i

    userdata Font > 5i {
        new (path=reg.g1: string) > 42i

    }
    fun FontDefault(name=reg.g1: string): Font > 43i
    fun FontUbuntuMono(): Font > 44i
    fun FontRoboto(): Font > 45i

    enum Events > 0i {
        Closed
        Resized
        LostFocus
        GainedFocus
        Input
        KeyPressed
        KeyReleased
        MouseWheelScrolled
        MouseButtonPressed
        MouseButtonReleased
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
    name: String,
    id: usize,
    lib_id: usize,
    gc_method: user_data::GcMethod,
}
impl Window {
    fn new(size: (u32, u32), title: &str, style: window::Style, settings: &WinBuilder) -> Self {
        let window = RenderWindow::new(size, title, style, &Default::default());
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

    fn cleanup(&mut self) {}
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

struct DrawStyle {
    color: Color,
    rotation: f32,
    scale: (f32, f32),
    outline_color: Color,
    outline_thickness: f32,
    font: Option<SfBox<Font>>,
    font_size: u32,
    character_spacing: f32,
    line_spacing: f32,
    name: String,
    id: usize,
    lib_id: usize,
    gc_method: user_data::GcMethod,
}
impl DrawStyle {
    /// Creates a new DrawStyle
    fn new() -> Self {
        Self {
            color: Color::WHITE,
            rotation: 0.0,
            scale: (1.0, 1.0),
            outline_color: Color::TRANSPARENT,
            outline_thickness: 0.0,
            font: None,
            font_size: 30,
            character_spacing: 1.0,
            line_spacing: 1.0,
            name: "DrawStyle".to_string(),
            id: 0,
            lib_id: 0,
            gc_method: user_data::GcMethod::None,
        }
    }
}
impl UserData for DrawStyle {
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
            style: window::Style::NONE,
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

    fn cleanup(&mut self) {}
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

struct Event {
    name: String,
    id: usize,
    lib_id: usize,
    gc_method: user_data::GcMethod,
    event: window::Event,
}
impl Event {
    fn new(event: window::Event) -> Self {
        Self {
            name: "Event".to_string(),
            id: 2,
            lib_id: 0,
            gc_method: user_data::GcMethod::None,
            event,
        }
    }
}
impl UserData for Event {
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
    name: String,
    id: usize,
    lib_id: usize,
    gc_method: user_data::GcMethod,
}
impl WinColor {
    fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r,
            g,
            b,
            a,
            name: "Color".to_string(),
            id: 3,
            lib_id: 0,
            gc_method: user_data::GcMethod::None,
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
            name: "Color".to_string(),
            id: 3,
            lib_id: 0,
            gc_method: user_data::GcMethod::None,
        }
    }
}
impl UserData for WinColor {
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
    name: String,
    id: usize,
    lib_id: usize,
    gc_method: user_data::GcMethod,
}
impl WinFont {
    fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            name: "Font".to_string(),
            id: 4,
            lib_id: 0,
            gc_method: user_data::GcMethod::None,
        }
    }
    fn default(name: &str) -> Self {
        let ruda_path = std::env::var("RUDA_PATH").unwrap_or_else(|_| ".".to_string());
        let path = format!("{}/fonts/{}.ttf", ruda_path, name);
        Self {
            path,
            name: "Font".to_string(),
            id: 4,
            lib_id: 0,
            gc_method: user_data::GcMethod::None,
        }
    }
    fn ubuntu_mono() -> Self {
        let ruda_path = std::env::var("RUDA_PATH").unwrap_or_else(|_| ".".to_string());
        let path = format!("{}/fonts/UbuntuMono-Regular.ttf", ruda_path);
        Self {
            path,
            name: "Font".to_string(),
            id: 4,
            lib_id: 0,
            gc_method: user_data::GcMethod::None,
        }
    }
    fn roboto() -> Self {
        let ruda_path = std::env::var("RUDA_PATH").unwrap_or_else(|_| ".".to_string());
        let path = format!("{}/fonts/Roboto-Regular.ttf", ruda_path);
        Self {
            path,
            name: "Font".to_string(),
            id: 4,
            lib_id: 0,
            gc_method: user_data::GcMethod::None,
        }
    }
}
impl UserData for WinFont {
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

    fn cleanup(&mut self) {}
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
