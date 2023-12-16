pub mod reader {
    use runtime::runtime::runtime_types::{self, Context, Instructions, Types};
    use std::fs::File;
    use std::io::prelude::*;
    struct Reader {
        pos: usize,
        ctx: runtime_types::Context,
        file: String,
    }
    pub fn read_file(path: String, context: Context) -> Context {
        let mut reader = Reader::new(context);
        let mut file =
            File::open(path.to_owned()).expect(&format!("File not found. ({})", path).to_owned());
        file.read_to_string(&mut reader.file)
            .expect("neco se pokazilo");
        while reader.file.as_bytes()[reader.pos] != '?' as u8 {
            reader.push_consts();
        }
        reader.pos += 1;
        while reader.file.as_bytes()[reader.pos] != '?' as u8 {
            reader.push_instruction();
        }
        reader.ctx.code.push(Instructions::End);
        reader.ctx
    }
    impl Reader {
        fn new(ctx: Context) -> Self {
            Reader {
                file: String::from(""),
                pos: 0,
                ctx,
            }
        }
        fn push_consts(&mut self) {
            self.pos += 1;
            let value = match self.file.as_bytes()[self.pos - 1] {
                65 => {
                    let bytes = self.read_bytes(8) as u32;
                    let int = unsafe { std::mem::transmute::<u32, i32>(bytes) };
                    Types::Int(int)
                }
                66 => {
                    let bytes = self.read_bytes(16) as u64;
                    let float = unsafe { std::mem::transmute::<u64, f64>(bytes) };
                    Types::Float(float)
                }
                67 => Types::Byte(self.read_unumber(2) as u8),
                68 => Types::Char((self.read_unumber(2) as u8) as char),
                69 => Types::Uint(self.read_unumber(32)),
                70 => Types::Bool(self.read_unumber(1) != 0),
                71 => Types::Null,
                34 => {
                    for idx in self.read_str_range() {
                        self.ctx
                            .stack
                            .push(Types::Char(self.file.as_bytes()[idx] as char));
                    }
                    self.ctx.stack.push(Types::Char('\0'));
                    return;
                }
                72 => todo!("Types::Enum(self.read_unumber(2) as u8)"),
                73 => Types::CodePointer(self.read_unumber(32) as usize),
                _ => {
                    panic!(
                        "Unexpected character '{}' at {}.",
                        self.file.as_bytes()[self.pos],
                        self.pos
                    )
                }
            };
            self.ctx.stack.push(value);
        }
        /// reads next instruction and pushes it to Context::code
        fn push_instruction(&mut self) {
            self.pos += 1;
            let instruction = match self.file.as_bytes()[self.pos - 1] {
                65 => Instructions::Wr(self.read_unumber(2)),
                66 => Instructions::Rd(self.read_unumber(2), self.read_unumber(1)),
                67 => Instructions::Wrp(self.read_unumber(1), self.read_unumber(1)),
                68 => Instructions::Rdp(self.read_unumber(1), self.read_unumber(1)),
                69 => Instructions::Rdc(self.read_unumber(2), self.read_unumber(1)),
                70 => Instructions::Ptr(self.read_unumber(2)),
                71 => Instructions::Alc(self.read_unumber(1), self.read_unumber(1)),
                72 => Instructions::Goto(self.read_unumber(4)),
                73 => Instructions::Brnc(self.read_unumber(4), self.read_unumber(4)),
                74 => Instructions::Ret,
                75 => Instructions::Res(self.read_unumber(2)),
                76 => Instructions::Mov(self.read_unumber(1), self.read_unumber(1)),
                77 => Instructions::Add,
                78 => Instructions::Sub,
                79 => Instructions::Mul,
                80 => Instructions::Div,
                81 => Instructions::Mod,
                82 => Instructions::Equ,
                83 => Instructions::Grt,
                84 => Instructions::And,
                85 => Instructions::Or,
                86 => Instructions::Not,
                87 => Instructions::Cal(self.read_unumber(3), self.read_unumber(2)),
                88 => Instructions::End,
                89 => Instructions::Dalc(self.read_unumber(1)),
                90 => Instructions::RAlc(self.read_unumber(1), self.read_unumber(1)),
                91 => Instructions::Idx(self.read_unumber(1), self.read_unumber(1)),
                92 => Instructions::Repp(self.read_unumber(1)),
                93 => Instructions::Less,
                94 => Instructions::Debug(self.read_unumber(1)),
                95 => Instructions::Gotop(self.read_unumber(1)),
                96 => Instructions::RRet,
                _ => {
                    panic!(
                        "Unexpected character '{}' at {}.",
                        self.file.as_bytes()[self.pos],
                        self.pos
                    )
                }
            };
            self.ctx.code.push(instruction);
        }
        fn read_str_range(&mut self) -> std::ops::Range<usize> {
            let bytes = self.file.as_bytes();
            let start = self.pos;
            while bytes[self.pos] != '"' as u8 {
                self.pos += 1;
            }
            self.pos += 1;
            start..(self.pos - 1)
        }
        fn read_bytes(&mut self, size: usize) -> u128 {
            let num = &self.file[self.pos..(self.pos + size)];
            self.pos += size;
            let bytes = u128::from_str_radix(num, 16).unwrap();
            bytes
        }
        fn read_unumber(&mut self, size: usize) -> usize {
            let num = &self.file[self.pos..(self.pos + size)];
            self.pos += size;
            usize::from_str_radix(num, 16).unwrap()
        }
    }
}
