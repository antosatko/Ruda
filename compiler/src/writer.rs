pub mod writer {
    use core::panic;

    use runtime::runtime::runtime_types::{Instructions, Types};
    pub fn write(code: &Vec<Instructions>, consts: &Vec<Types>, file_name: &str) {
        use std::fs::File;
        use std::io::prelude::*;
        let mut file = File::create(file_name).expect("nevim");
        file.write_all(to_string(code, consts).as_bytes())
            .expect("furt nevim");
    }
    pub fn to_string(code: &Vec<Instructions>, consts: &Vec<Types>) -> String {
        let mut str = String::new();
        let mut i = 0;
        while i < consts.len() {
            if let Types::Char(_) = consts[i] {
                if let Some(string) = get_str(i, &consts) {
                    str.push_str(&string);
                    i += string.len() - 1;
                } else {
                    str.push_str(&val_to_string(consts[i]));
                    i += 1;
                }
            } else {
                str.push_str(&val_to_string(consts[i]));
                i += 1;
            }
        }
        str.push_str("?");
        for instr in code.iter() {
            str.push_str(&instr_to_str(*instr))
        }
        str.push_str("?");
        str
    }
    pub fn get_str(mut index: usize, consts: &Vec<Types>) -> Option<String> {
        let mut str = String::from("\"");
        while let Types::Char(char) = consts[index] {
            if index >= consts.len() {
                break;
            }
            if char == '\0' {
                str.push('"');
                return Some(str);
            }
            index += 1;
            str.push(char);
        }
        None
    }
    pub fn val_to_string(val: Types) -> String {
        use Types::*;
        match val {
            Int(int) => {
                let bytes = unsafe { std::mem::transmute::<i32, u32>(int) };
                format!("{}{:8x}", 65 as char, bytes).replace(" ", "0")
            }
            Float(float) => {
                let bytes = unsafe { std::mem::transmute::<f64, u64>(float) };
                format!("{}{:16x}", 66 as char, bytes).replace(" ", "0")
            }
            Byte(byte) => format!("{}{byte:2x}", 67 as char).replace(" ", "0"),
            Char(char) => format!("{}{:2x}", 68 as char, char as u8).replace(" ", "0"),
            Uint(usize) => format!("{}{usize:32x}", 69 as char).replace(" ", "0"),
            Bool(bool) => {
                let num = if bool { 1 } else { 0 };
                format!("{}{:1x}", 70 as char, num).replace(" ", "0")
            }
            Pointer(_, _) => String::new(),
            Null => format!("{}", 71 as char),
            //Enum(offset) => format!("{}{offset:2x}", 72 as char).replace(" ", "0"),
            CodePointer(u_size) => format!("{}{u_size:32x}", 73 as char).replace(" ", "0"),
        }
    }
    pub fn instr_to_str(instr: Instructions) -> String {
        use Instructions::*;
        macro_rules! bin_instr {
            ($code: literal) => {
                format!("{}", $code as char)
            };
            ($code: literal, ($n: ident, $bytes: literal)) => {
                format!("{}{}", $code as char, num_to_hbytes($n, $bytes))
            };
            ($code: literal, ($n: ident, $bytes: literal), ($n1: ident, $bytes1: literal)) => {
                format!(
                    "{}{}{}",
                    $code as char,
                    num_to_hbytes($n, $bytes),
                    num_to_hbytes($n1, $bytes1)
                )
            };
        }
        match instr {
            Wr(n) => bin_instr!(65, (n, 2)),
            Rd(n, n1) => bin_instr!(66, (n, 2), (n1, 1)),
            Wrp(n, n1) => bin_instr!(67, (n, 1), (n1, 1)),
            Rdp(n, n1) => bin_instr!(68, (n, 1), (n1, 1)),
            Rdc(n, n1) => bin_instr!(69, (n, 2), (n1, 1)),
            Ptr(n) => bin_instr!(70, (n, 2)),
            Alc(n, n1) => bin_instr!(71, (n, 1), (n1, 1)),
            Goto(n) => bin_instr!(72, (n, 4)),
            Brnc(n, n1) => bin_instr!(73, (n, 4), (n1, 4)),
            Ret => bin_instr!(74),
            Res(n) => bin_instr!(75, (n, 2)),
            Mov(n, n1) => bin_instr!(76, (n, 1), (n1, 1)),
            Add => bin_instr!(77),
            Sub => bin_instr!(78),
            Mul => bin_instr!(79),
            Div => bin_instr!(80),
            Mod => bin_instr!(81),
            Equ => bin_instr!(82),
            Grt => bin_instr!(83),
            And => bin_instr!(84),
            Or => bin_instr!(85),
            Not => bin_instr!(86),
            Cal(n, n1) => bin_instr!(87, (n, 3), (n1, 2)),
            End => bin_instr!(88),
            Dalc(n) => bin_instr!(89, (n, 1)),
            RAlc(n, n1) => bin_instr!(90, (n, 1), (n1, 1)),
            Idx(n, n1) => bin_instr!(91, (n, 1), (n1, 1)),
            Repp(n) => bin_instr!(92, (n, 1)),
            Less => bin_instr!(93),
            Debug(n) => bin_instr!(94, (n, 1)),
            Gotop(n) => bin_instr!(95, (n, 4)),
            RRet => bin_instr!(96),
            Cast(n, n1) => bin_instr!(97, (n, 1), (n1, 1)),
            Len(n) => bin_instr!(98, (n, 1)),
            Type(n, n1) => bin_instr!(99, (n, 1), (n1, 1)),
        }
    }
    pub fn num_to_hbytes(num: usize, bytes: u8) -> String {
        match bytes {
            1 => format!("{:1x}", num),
            2 => format!("{:2x}", num),
            3 => format!("{:3x}", num),
            4 => format!("{:4x}", num),
            _ => panic!("{}", bytes),
        }
        .replace(" ", "0")
    }
}
