//! Simple library used to draw shapes on canvas.
//! 
//! Aiming to help students learn 2D computer graphics.
//! 
//! Contains:
//!  - 2 separate buffers for drawing.
//!  - basic shapes (rectangle, line, circle, point).
//!  - transparent colors, gradients, filters, ...
//! 
//! Module is my open source project.
//! It is not intended for real-world projects due to its lack of features and need for external library for drawing on screen.
//! But rather for CS students who want to explore 2D graphics and learn something new. Editing source code is highly encouraged.

/// Canvas context and its methods.
#[allow(unused)]
pub mod canvas {
    #[derive(Clone)]
    /// Canvas context.
    pub struct Context {
        /// Resolution (X, Y)
        pub port: (usize, usize),
        /// Displayed buffer
        pub front: Vec<Vec<Pixel>>,
        /// Buffer to draw on
        pub back: Vec<Vec<Pixel>>,
        line_thickness: usize,
    }

    impl Context {
        /// Canvas constructor.
        /// Example
        /// ```
        /// use canvas::canvas::*;
        /// let mut ctx = Context::new().view_port(150, 150).build();
        /// ctx.fill(Shape::Rect(50,50,50,50), Colors::Red);
        /// 
        /// ctx.swap();
        /// // canvas does not provide drawing on the screen.
        /// // use your desired method.
        /// ```
        pub fn new() -> Context {
            Context {
                port: (0, 0),
                front: Vec::new(),
                back: Vec::new(),
                line_thickness: 0,
            }
        }
        /// Constructor method for resolution.
        /// Can also be used for resizing existing context.
        /// 
        /// Example:
        /// ```
        /// let mut ctx = Context::new().view_port(150, 150).build();
        /// ctx.view_port(200, 200);
        /// 
        /// assert_eq!(200, canvas.port.0);
        /// ```
        pub fn view_port(&mut self, x: usize, y: usize) -> &mut Self {
            self.port = (x, y);
            self.back = Vec::new();
            self.front = Vec::new();
            for i in 0..x {
                self.back.push(Vec::new());
                self.front.push(Vec::new());
                for _ in 0..y {
                    self.back[i].push(Pixel::from_color(Colors::Black));
                    self.front[i].push(Pixel::from_color(Colors::Black));
                }
            }
            self
        }
        /// Constructor method to build new context.
        pub fn build(&mut self) -> Self {
            self.to_owned()
        }
        /// Clears context using passed color.
        pub fn clear(&mut self, color: Colors) {
            for i in 0..self.port.0 {
                for j in 0..self.port.1 {
                    self.front[i][j] = Pixel::from_color(color);
                    self.back[i][j] = Pixel::from_color(color);
                }
            }
        }
        /// Changes color of entered pixel depending on supplied filter.
        /// Pixels offset does not affect finall product.
        fn apply_filter(&mut self, x: i32, y: i32, filter: &Filters) {
            if x < self.port.0 as i32 && y < self.port.1 as i32 && x >= 0 && y >= 0 {
                match filter {
                    // ik its bad
                    Filters::Gamma(value) => {
                        let pixref = &mut self.back[x as usize][y as usize];
                        let valsqrt = f64::sqrt(*value as f64);
                        pixref.0 = (f64::sqrt(pixref.0 as f64) * valsqrt) as u8;
                        pixref.1 = (f64::sqrt(pixref.1 as f64) * valsqrt) as u8;
                        pixref.2 = (f64::sqrt(pixref.2 as f64) * valsqrt) as u8;
                    }
                    Filters::Color(color) => {
                        let pixref = &mut self.back[x as usize][y as usize];
                        let pix = Pixel::from_color(*color);
                        pixref.0 = ((pix.0 as f64 / 255f64) * pixref.0 as f64) as u8;
                        pixref.1 = ((pix.1 as f64 / 255f64) * pixref.1 as f64) as u8;
                        pixref.2 = ((pix.2 as f64 / 255f64) * pixref.2 as f64) as u8;
                    }
                    _ => {}
                }
            }
        }
        /// Applies filter on provided shape.
        pub fn filter(&mut self, shape: Shape, filter: Filters) {
            match shape {
                Shape::Rect(mut x, mut y, mut w, mut h) => {
                    if w < 0 {
                        w *= -1;
                        x -= w;
                    }
                    if h < 0 {
                        h *= -1;
                        y -= h;
                    }
                    for i in x..(w + x) {
                        for j in y..(h + y) {
                            self.apply_filter(i, j, &filter);
                        }
                    }
                }
                Shape::Arc(x, y, r) => {
                    let c = r * r;
                    for i in 0..(r * 2) {
                        self.apply_filter(x, (y as f64 + i as f64 - r as f64 + 1.) as i32, &filter);
                    }
                    for i in 1..r {
                        let height = f64::sqrt((c - i * i) as f64);
                        for j in 0..(f64::ceil(height * 2.) as i32) {
                            self.apply_filter(
                                x + i,
                                (y as f64 + j as f64 - height as f64 + 1.) as i32,
                                &filter,
                            );
                            self.apply_filter(
                                x - i,
                                (y as f64 + j as f64 - height as f64 + 1.) as i32,
                                &filter,
                            );
                        }
                    }
                }
                _ => (),
            }
        }
        /// Fills shape with provided color.
        pub fn fill(&mut self, shape: Shape, color: Colors) {
            match shape {
                Shape::Rect(mut x, mut y, mut w, mut h) => {
                    if w < 0 {
                        w *= -1;
                        x -= w;
                    }
                    if h < 0 {
                        h *= -1;
                        y -= h;
                    }
                    for i in x..(w + x) {
                        for j in y..(h + y) {
                            self.set_pixel(i, j, Pixel::from_color(color));
                        }
                    }
                }
                Shape::Arc(x, y, r) => {
                    let c = r * r;
                    for i in 0..(r * 2) {
                        self.set_pixel(
                            x,
                            (y as f64 + i as f64 - r as f64 + 1.) as i32,
                            Pixel::from_color(color),
                        );
                    }
                    for i in 1..r {
                        let height = f64::sqrt((c - i * i) as f64);
                        for j in 0..(f64::ceil(height * 2.) as i32) {
                            self.set_pixel(
                                x + i,
                                (y as f64 + j as f64 - height as f64 + 1.) as i32,
                                Pixel::from_color(color),
                            );
                            self.set_pixel(
                                x - i,
                                (y as f64 + j as f64 - height as f64 + 1.) as i32,
                                Pixel::from_color(color),
                            );
                        }
                    }
                }
                _ => (),
            }
        }
        /// Tries to return value of asked pixel.
        fn get_pixel(&mut self, x: i32, y: i32) -> Option<Pixel> {
            return if x < self.port.0 as i32 && y < self.port.1 as i32 && x >= 0 && y >= 0 {
                //Some(self.back[x as usize][y as usize])
                Some(self.back[x as usize][y as usize])
            } else {
                None
            };
        }
        /// Sets color of pixel, or applies transparent color.
        fn set_pixel(&mut self, x: i32, y: i32, pixel: Pixel) {
            if x < self.port.0 as i32 && y < self.port.1 as i32 && x >= 0 && y >= 0 {
                self.back[x as usize][y as usize] = match pixel.3 {
                    255 => pixel,
                    _ => Pixel(
                        (pixel.3 as f32 * (pixel.0 as f32 / 255.)
                            + (1. - pixel.3 as f32 / 255.)
                                * self.back[x as usize][y as usize].0 as f32)
                            as u8,
                        (pixel.3 as f32 * (pixel.1 as f32 / 255.)
                            + (1. - pixel.3 as f32 / 255.)
                                * self.back[x as usize][y as usize].1 as f32)
                            as u8,
                        (pixel.3 as f32 * (pixel.2 as f32 / 255.)
                            + (1. - pixel.3 as f32 / 255.)
                                * self.back[x as usize][y as usize].2 as f32)
                            as u8,
                        255,
                    ),
                };
            }
        }
        /// Corrects gradient verticies provided by user.
        fn preprocess_gradient(gradient: &mut Gradient) -> Vec<Pixel> {
            fn bubblesort(verticies: &mut Vec<(Colors, f32)>) {
                let mut i = 0;
                while i < verticies.len() {
                    let mut j = 0;
                    while j < verticies.len() - 1 {
                        if verticies[j].1 > verticies[j + 1].1 {
                            let med = verticies[j];
                            verticies[j] = verticies[j + 1];
                            verticies[j + 1] = med;
                        }
                        j += 1;
                    }
                    i += 1;
                }
            }
            let mut cols: Vec<Pixel> = Vec::new();
            match gradient {
                Gradient::Line(verticies) => {
                    bubblesort(verticies);
                    for vert in verticies.iter() {
                        cols.push(Pixel::from_color(vert.0));
                    }
                    cols
                }
                Gradient::OrientedLine(_, verticies) => {
                    bubblesort(verticies);
                    for vert in verticies.iter() {
                        cols.push(Pixel::from_color(vert.0));
                    }
                    cols
                }
                Gradient::Plane(verticies, _) => {
                    for vert in verticies.iter() {
                        cols.push(Pixel::from_color(vert.0));
                    }
                    cols
                }
                Gradient::OrientedPoint(_, verticies) => {
                    bubblesort(verticies);
                    for vert in verticies.iter() {
                        cols.push(Pixel::from_color(vert.0));
                    }
                    cols
                }
                Gradient::Point(verticies) => {
                    for vert in verticies.iter() {
                        cols.push(Pixel::from_color(vert.0));
                    }
                    cols
                }
                Gradient::OrientedPlane(verticies, background) => {
                    for vert in verticies.iter() {
                        cols.push(Pixel::from_color(vert.0));
                    }
                    cols.push(Pixel::from_color(*background));
                    cols
                }
            }
        }
        /// Draws gradient colors on provided shape.
        pub fn gradient(&mut self, shape: Shape, mut grad: Gradient) {
            let colors = Self::preprocess_gradient(&mut grad);
            match shape {
                Shape::Arc(x, y, r) => {
                    let c = r * r;
                    for i in 0..r {
                        let height = f64::sqrt((c - i * i) as f64);
                        for j in 0..(f64::ceil(height * 2.) as i32) {
                            let pix = Pixel::from_grad(
                                &grad,
                                (
                                    (r + i) as f32 / (r as f32 * 2.),
                                    (r - height as i32 + j) as f32 / (r as f32 * 2.),
                                ),
                                &colors,
                            );
                            self.set_pixel(
                                x + i,
                                (y as f64 + j as f64 - height as f64) as i32,
                                pix,
                            );
                            self.set_pixel(
                                x - i,
                                (y as f64 + j as f64 - height as f64) as i32,
                                Pixel::from_grad(
                                    &grad,
                                    (
                                        (r - i) as f32 / (r as f32 * 2.),
                                        (r - height as i32 + j) as f32 / (r as f32 * 2.),
                                    ),
                                    &colors,
                                ),
                            );
                        }
                    }
                }
                Shape::Line(x1, y1, x2, y2) => {
                    let dx = x2 - x1;
                    let dy = y2 - y1;
                    let sx = if dx != 0 { dx / dx.abs() } else { 1 }; // s jako smeti
                    let sy = if dy != 0 { dy / dy.abs() } else { 1 };
                    let x: f64 = dx as f64 / dy as f64;
                    let y: f64 = dy as f64 / dx as f64;
                    for i in 0..dx.abs() {
                        let smetx = i * sx + x1;
                        let smety = (y * (i * sx) as f64 + y1 as f64) as i32;
                        self.set_pixel(
                            smetx,
                            smety,
                            Pixel::from_grad(&grad, (i as f32 / dx.abs() as f32, 0.), &colors),
                        );
                    }
                    for i in 0..dy.abs() {
                        let smetx = (x * (i * sy) as f64 + x1 as f64) as i32;
                        let smety = i * sy + y1;
                        self.set_pixel(
                            smetx,
                            smety,
                            Pixel::from_grad(&grad, (i as f32 / dx.abs() as f32, 0.), &colors),
                        );
                    }
                }
                Shape::Rect(mut x, mut y, mut w, mut h) => {
                    if w < 0 {
                        w *= -1;
                        x -= w;
                    }
                    if h < 0 {
                        h *= -1;
                        y -= h;
                    }
                    for i in 0..w {
                        for j in 0..h {
                            self.set_pixel(
                                i + x,
                                j + y,
                                Pixel::from_grad(
                                    &grad,
                                    (i as f32 / w as f32, j as f32 / h as f32),
                                    &colors,
                                ),
                            );
                        }
                    }
                }
                _ => todo!(),
            }
        }
        /// Check if pixel is not out of bounds.
        fn legal_pixel(&mut self, x: i32, y: i32) -> bool {
            x < self.port.0 as i32 && y < self.port.1 as i32 && x > 0 && y > 0
        }
        /// Outlines provided Shape.
        pub fn draw(&mut self, shape: Shape, color: Colors) {
            //use std::num;
            match shape {
                Shape::Rect(mut x, mut y, mut w, mut h) => {
                    if w < 0 {
                        w *= -1;
                        x -= w;
                    }
                    if h < 0 {
                        h *= -1;
                        y -= h;
                    }
                    for i in x..(w + x + 1) {
                        self.set_pixel(x, i, Pixel::from_color(color));
                        for j in 0..self.line_thickness {
                            self.set_pixel(i, y + j as i32, Pixel::from_color(color));
                            self.set_pixel(i, y - j as i32, Pixel::from_color(color));
                            self.set_pixel(i, y + j as i32 + h, Pixel::from_color(color));
                            self.set_pixel(i, y + j as i32 + h, Pixel::from_color(color));
                        }
                        self.set_pixel(i, y + h, Pixel::from_color(color));
                    }
                    for i in x..(w + x + 1) {
                        self.set_pixel(x, i, Pixel::from_color(color));
                        for j in 0..self.line_thickness {
                            self.set_pixel(x + j as i32, i, Pixel::from_color(color));
                            self.set_pixel(x - j as i32, i, Pixel::from_color(color));
                            self.set_pixel(x + j as i32 + w, i, Pixel::from_color(color));
                            self.set_pixel(x - j as i32 + w, i, Pixel::from_color(color));
                        }
                        self.set_pixel(x + w, i, Pixel::from_color(color));
                    }
                }
                Shape::Point(x, y) => {
                    self.set_pixel(x, y, Pixel::from_color(color));
                }
                Shape::Line(x1, y1, x2, y2) => {
                    let a = x2 - x1;
                    let b = y2 - y1;
                    let c = f64::sqrt((a * a + b * b) as f64);
                    let fa = a as f64 / c;
                    let fb = b as f64 / c;
                    let pix = Pixel::from_color(color);
                    for i in 0..(c.ceil() as i32 + 1) {
                        self.set_pixel(
                            (fa * i as f64) as i32 + x1,
                            (fb * i as f64) as i32 + y1,
                            pix,
                        );
                    }
                }
                Shape::Arc(x, y, r) => {
                    let c = r * r;
                    for i in 0..r {
                        let height = f64::sqrt((c - i * i) as f64).ceil() as i32;
                        self.set_pixel(x + i, y + height, Pixel::from_color(color));
                        self.set_pixel(x - i, y + height, Pixel::from_color(color));
                        self.set_pixel(x - i, y - height, Pixel::from_color(color));
                        self.set_pixel(x + i, y - height, Pixel::from_color(color));
                        self.set_pixel(x + height, y + i, Pixel::from_color(color));
                        self.set_pixel(x - height, y + i, Pixel::from_color(color));
                        self.set_pixel(x - height, y - i, Pixel::from_color(color));
                        self.set_pixel(x + height, y - i, Pixel::from_color(color));
                    }
                }
            }
        }
        /// Swaps front and back buffer for drawing new frame.
        pub fn swap(&mut self) {
            let medium: Vec<Vec<Pixel>> = self.front.to_owned();
            self.front = self.back.to_owned();
            self.back = medium.to_owned();
        }
    }

    impl Pixel {
        /// Constructor returns new untransparent Pixel.
        pub fn from_rgb(r: u8, g: u8, b: u8) -> Pixel {
            Pixel(r, g, b, 255)
        }
        /// Construcor returns new Pixel using Colors enum.
        pub fn from_color(color: Colors) -> Pixel {
            match color {
                Colors::Red => Pixel(255, 0, 0, 255),
                Colors::Green => Pixel(0, 255, 0, 255),
                Colors::Yellow => Pixel(255, 255, 0, 255),
                Colors::Purple => Pixel(255, 0, 255, 255),
                Colors::Blue => Pixel(0, 0, 255, 255),
                Colors::Black => Pixel(0, 0, 0, 255),
                Colors::Aqua => Pixel(0, 255, 255, 255),
                Colors::White => Pixel(255, 255, 255, 255),
                Colors::Gray => Pixel(128, 128, 128, 255),
                Colors::RGB(r, g, b) => Pixel(r, g, b, 255),
                Colors::RGBA(r, g, b, a) => Pixel(r, g, b, a),
                Colors::Hue(_) => Pixel(0, 0, 0, 0),
            }
        }
        /// Constructor returns new Pixel using gradient colors and its offset from the middle.
        fn from_grad(grad: &Gradient, position: (f32, f32), colors: &Vec<Pixel>) -> Pixel {
            match grad {
                Gradient::Line(verticies) => {
                    //position.0 = 1. - position.0;
                    let mut i = 0;
                    let right;
                    let left = loop {
                        if verticies[i].1 <= position.0 && verticies[i + 1].1 >= position.0
                            || i == verticies.len() - 2
                        {
                            right = (colors[i + 1], verticies[i + 1].1 - position.0);
                            break (colors[i], position.0 - verticies[i].1);
                        }
                        i += 1;
                    };
                    let max = left.1 + right.1;
                    let multipliers = (right.1 / max, left.1 / max);
                    Pixel::from_rgb(
                        (left.0 .0 as f32 * multipliers.0 + right.0 .0 as f32 * multipliers.1)
                            as u8,
                        (left.0 .1 as f32 * multipliers.0 + right.0 .1 as f32 * multipliers.1)
                            as u8,
                        (left.0 .2 as f32 * multipliers.0 + right.0 .2 as f32 * multipliers.1)
                            as u8,
                    )
                }
                Gradient::Plane(verticies, brightness) => {
                    let mut cols = (0., 0., 0.);
                    let mut count = 0.;
                    for (i, vert) in verticies.iter().enumerate() {
                        let dist = {
                            let a = position.0 - vert.1;
                            let b = position.1 - vert.2;
                            f32::sqrt((a * a + b * b) as f32)
                        };
                        cols.0 += colors[i].0 as f32 / dist * brightness;
                        cols.1 += colors[i].1 as f32 / dist * brightness;
                        cols.2 += colors[i].2 as f32 / dist * brightness;
                        count += dist;
                    }
                    cols.0 /= count;
                    cols.1 /= count;
                    cols.2 /= count;
                    Pixel::from_rgb(cols.0 as u8, cols.1 as u8, cols.2 as u8)
                }
                Gradient::OrientedPlane(verticies, _) => {
                    let mut cols = {
                        let bg = colors[colors.len() - 1];
                        (bg.0 as f32, bg.1 as f32, bg.2 as f32, bg.3)
                    };
                    let mut count = 0.;
                    for (i, vert) in verticies.iter().enumerate() {
                        let dist = {
                            let a = position.0 - vert.1;
                            let b = position.1 - vert.2;
                            f32::sqrt((a * a + b * b) as f32)
                        };
                        cols.0 += colors[i].0 as f32 / dist * verticies[i].3;
                        cols.1 += colors[i].1 as f32 / dist * verticies[i].3;
                        cols.2 += colors[i].2 as f32 / dist * verticies[i].3;
                        count += dist;
                    }
                    cols.0 /= count;
                    cols.1 /= count;
                    cols.2 /= count;
                    Pixel(cols.0 as u8, cols.1 as u8, cols.2 as u8, cols.3)
                }
                Gradient::Point(verticies) => {
                    let mut i = 0;
                    let dist_from_center = {
                        let a = position.0 - 0.5;
                        let b = position.1 - 0.5;
                        (a * a + b * b).sqrt()
                    };
                    let right;
                    let left = loop {
                        if verticies[i].1 <= dist_from_center
                            && verticies[i + 1].1 >= dist_from_center
                            || i == verticies.len() - 2
                        {
                            right = (colors[i + 1], verticies[i + 1].1 - dist_from_center);
                            break (colors[i], dist_from_center - verticies[i].1);
                        }
                        i += 1;
                    };
                    let max = left.1 + right.1;
                    let multipliers = (right.1 / max, left.1 / max);
                    Pixel::from_rgb(
                        (left.0 .0 as f32 * multipliers.0 + right.0 .0 as f32 * multipliers.1)
                            as u8,
                        (left.0 .1 as f32 * multipliers.0 + right.0 .1 as f32 * multipliers.1)
                            as u8,
                        (left.0 .2 as f32 * multipliers.0 + right.0 .2 as f32 * multipliers.1)
                            as u8,
                    )
                }
                Gradient::OrientedPoint(center, verticies) => {
                    let mut i = 0;
                    let dist_from_center = {
                        let a = position.0 - center.0;
                        let b = position.1 - center.1;
                        (a * a + b * b).sqrt()
                    };
                    let right;
                    let left = loop {
                        if verticies[i].1 <= dist_from_center
                            && verticies[i + 1].1 >= dist_from_center
                            || i == verticies.len() - 2
                        {
                            right = (colors[i + 1], verticies[i + 1].1 - dist_from_center);
                            break (colors[i], dist_from_center - verticies[i].1);
                        }
                        i += 1;
                    };
                    let max = left.1 + right.1;
                    let multipliers = (right.1 / max, left.1 / max);
                    Pixel::from_rgb(
                        (left.0 .0 as f32 * multipliers.0 + right.0 .0 as f32 * multipliers.1)
                            as u8,
                        (left.0 .1 as f32 * multipliers.0 + right.0 .1 as f32 * multipliers.1)
                            as u8,
                        (left.0 .2 as f32 * multipliers.0 + right.0 .2 as f32 * multipliers.1)
                            as u8,
                    )
                }
                _ => todo!(),
            }
        }
    }
    /// Shapes accepted by context methods
    pub enum Shape {
        /// x y w h
        Rect(i32, i32, i32, i32),
        /// x y
        Point(i32, i32),
        /// x1 y1 x2 y2
        Line(i32, i32, i32, i32),
        /// x y r
        Arc(i32, i32, i32),
    }

    #[derive(Clone, Copy)]
    /// Pixel used for inner computing.
    pub struct Pixel(pub u8, pub u8, pub u8, pub u8);
    #[derive(Clone, Copy)]
    /// Premade colors.
    pub enum Colors {
        Red,
        Green,
        Blue,
        Black,
        White,
        Gray,
        Yellow,
        Purple,
        Aqua,
        RGB(u8, u8, u8),
        RGBA(u8, u8, u8, u8),
        Hue(i32),
    }

    /// Usable gradient types.
    /// Oriented types offer more control over end result.
    pub enum Gradient {
        /// Gradient values drawn from single point.
        /// Usually convenient for drawing the sun.
        /// ```
        /// ctx.gradient(Shapes::rect(0,0,150,150), 
        ///     Gradient::Point(vec![(Colors::Yellow, 0.), (Colors::Blue, 0.3)])
        /// );
        /// // WOW! What a nice sun!
        /// ```
        Point(Vec<(Colors, f32)>),
        /// Gradient values drawn from single point with a specified location.
        /// Usually convenient for drawing the sun.
        /// ```
        /// ctx.gradient(Shapes::rect(0,0,150,150), 
        ///     Gradient::OrientedPoint((0.1, 0.1),
        ///     vec![(Colors::Yellow, 0.), (Colors::Blue, 0.3)]
        /// ));
        /// // WOW! What a nice sun in the corner!
        /// ```
        OrientedPoint((f32, f32), Vec<(Colors, f32)>),
        /// Also gradient 1D does exacly what you would expect.
        /// ```
        /// ctx.gradient(Shapes::rect(0,0,150,150), 
        ///     Gradient::Line(vec![(Colors::Yellow, 0.), (Colors::Blue, 0.3), (Colors::Red, 1.)])
        /// );
        /// // WOW! What a nice stripe of colors!
        /// ```
        Line(Vec<(Colors, f32)>),
        /// Does not work atm.
        OrientedLine((f32, f32, f32, f32), Vec<(Colors, f32)>),
        /// This one is not exacly gradient but I liked how it looks. So here it is. 
        /// Basicly makes a plane full of fancy lights. 
        /// Feel free to try it out.
        /// ```
        /// // vec(color, x, y), color_brightness
        /// ctx.gradient(Shapes::rect(0,0,150,150), 
        ///     Gradient::Plane(
        ///         vec![(Colors::Yellow, 0., 0.), (Colors::Blue, 0.3, 0.3), (Colors::Red, 1., 0.5)],
        ///         0.3
        /// ));
        /// // WOW! Such cool good-looking lights!
        /// ```
        Plane(Vec<(Colors, f32, f32)>, f32),
        /// The same as normal verison with control over individual light sizes and background color + transparency
        /// ```
        /// // vec(color, x, y, brightness), background
        /// ctx.gradient(Shapes::rect(0,0,150,150), 
        ///     Gradient::Plane(
        ///         vec![(Colors::Yellow, 0., 0., 0.2), 
        ///             (Colors::Blue, 0.3, 0.3, 0.2), 
        ///             (Colors::Red, 1., 0.5, 1.)
        ///         ],
        ///         Colors::Blue
        /// ));
        /// // WOW! Such cool good-looking nicely moderated lights! I am impressed!
        /// ```
        OrientedPlane(Vec<(Colors, f32, f32, f32)>, Colors),
    }

    /// Future feature.
    pub enum Effects {
        // radius
        Blur(i32),
        // mask, center
        OrientedBlur(Vec<Vec<i32>>, i32),
        // radius, color
        EdgeDetection(i32, Colors),
        // radius, center, color
        OrientedEdgeDetection(Vec<Vec<i32>>, i32, Colors),
    }

    /// Simpliest way of modifing image.
    /// Filters can not interact with nearby pixels or offsets.
    pub enum Filters {
        Color(Colors),
        Gamma(u8),
        Contrast(u8),
        Temperature(u8),
    }
}
