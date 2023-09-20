pub mod lexing_preprocessor {

    use std::time::SystemTime;

    use crate::{lexer::tokenizer::{deparse_token, Operators, Tokens}, tree_walker::tree_walker::Line};

    use super::parse_err::Errors;
    pub fn refactor(
        mut tokens: Vec<Tokens>,
        lines: Vec<(usize, usize)>,
        errors: &mut Vec<Errors>,
    ) -> Result<(Vec<Tokens>, Vec<(usize, usize)>), LexingErr> {
        let mut i = 0;
        tokens.push(Tokens::EndOfFile);
        while i < tokens.len() {
            i += process_token(&mut tokens, i, &lines, errors);
        }
        Ok(clear(&tokens, &lines))
    }
    fn process_token(
        tokens: &mut Vec<Tokens>,
        idx: usize,
        lines: &Vec<(usize, usize)>,
        errors: &mut Vec<Errors>,
    ) -> usize {
        match &tokens[idx] {
            Tokens::Text(txt) => {
                let mut chars = txt.chars();
                let first = chars.next();
                let last = if let Some(last) = chars.last() {last} else {first.unwrap()};
                if let Some(first) = first {
                    if first.is_ascii_digit() {
                        // float
                        if let Tokens::Dot = &tokens[idx + 1] {
                            let first_num = if let Ok(num) = txt.parse::<usize>() {
                                num
                            } else {
                                // syntax err: incorrect number
                                errors.push(Errors::InvalidNumber(Line::from(lines[idx]), txt.to_string()));
                                return 1;
                            };
                            if let Tokens::Text(txt2) = &tokens[idx + 2] {
                                let mut float = String::from("0.");
                                float.push_str(txt2);
                                if let Ok(num2) = float.parse::<f64>() {
                                    tokens[idx] = Tokens::Number(first_num as f64 + num2, 'f');
                                    remove(tokens, idx + 1);
                                    remove(tokens, idx + 2);
                                    return 2;
                                } else {
                                    // syntax err: incorrect number
                                    let mut res = txt.to_string();
                                    res.push('.');
                                    res.push_str(txt2);
                                    errors.push(Errors::InvalidNumber(Line::from(lines[idx]), res));
                                    return 1;
                                };
                            } else {
                                // syntax err: unexpected symbol: .
                            }
                        // int
                        } else {
                            if last.is_ascii_digit() {
                                if let Ok(num) = txt.parse::<usize>() {
                                    tokens[idx] = Tokens::Number(num as f64, 'n')
                                } else {
                                    errors.push(Errors::InvalidNumber(Line::from(lines[idx]), txt.to_string()));
                                    // syntax err: incorrect number
                                }
                            } else {
                                if let Ok(num) = txt[..txt.len() - 1].parse::<usize>() {
                                    tokens[idx] =
                                        Tokens::Number(num as f64, last)
                                } else {
                                    errors.push(Errors::InvalidNumber(Line::from(lines[idx]), txt.to_string()));
                                    // syntax err: incorrect number
                                }
                            }
                        }
                        return 1;
                    }
                }
                for char in txt.chars() {
                    if !char.is_whitespace() {
                        return 1;
                    }
                }
                // is whitespace
                remove(tokens, idx);
                return 1;
            }
            Tokens::Whitespace(_) => {
                remove(tokens, idx);
                return 1;
            }
            Tokens::DoubleQuotes => {
                let mut i = idx + 1;
                let mut res = String::new();
                while tokens[i] != Tokens::DoubleQuotes {
                    res.push_str(&deparse_token(&tokens[i]));
                    i += 1;
                    if i == tokens.len() {
                        // syntax err: end of string never found
                        remove_range(tokens, idx + 1, tokens.len());
                        tokens[idx] = Tokens::String(res);
                        return 1;
                    }
                }
                remove_range(tokens, idx + 1, i + 1);
                tokens[idx] = Tokens::String(res);
            }
            Tokens::Quotes => {
                let mut i = idx + 1;
                let mut res = String::from("\"");
                while tokens[i] != Tokens::Quotes {
                    res.push_str(&deparse_token(&tokens[i]));
                    i += 1;
                    if i == tokens.len() {
                        // syntax err: end of string never found
                        remove_range(tokens, idx + 1, tokens.len());
                        tokens[idx] = Tokens::String(res);
                        return 1;
                    }
                }
                remove_range(tokens, idx + 1, i + 1);
                res.push('"');
                let temp = snailquote::unescape(&res);
                match temp {
                    Ok(temp) => {
                        if temp.len() == 1 {
                            tokens[idx] = Tokens::Char(temp.chars().next().unwrap());
                        } else {
                            errors.push(Errors::CharacterTooLong(Line::from(lines[idx]), temp));
                        }
                    }
                    Err(err) => {
                        // syntax err: invalid string
                        errors.push(Errors::InvalidChar(Line::from(lines[idx]), err.to_string()));
                        tokens[idx] = Tokens::String(res);
                    }
                }
            }
            Tokens::Space => {
                remove(tokens, idx);
            }
            Tokens::Colon => {
                if !not_end(idx, &tokens) {
                    return 1;
                }
                if let Tokens::Colon = tokens[idx + 1] {
                    tokens[idx] = Tokens::DoubleColon;
                    remove(tokens, idx + 1);
                }
                return 1;
            }
            Tokens::Semicolon => {
                if tokens.len() != idx + 1 {
                    while let Tokens::Semicolon = tokens[idx + 1] {
                        remove(tokens, idx + 1);
                    }
                }
            }
            Tokens::Operator(op) => match op {
                Operators::Plus => {
                    if let Tokens::Operator(Operators::Equal) = tokens[idx + 1] {
                        tokens[idx] = Tokens::Operator(Operators::AddEq);
                        remove(tokens, idx + 1);
                    }
                }
                Operators::Minus => {
                    if let Tokens::Operator(Operators::Equal) = tokens[idx + 1] {
                        tokens[idx] = Tokens::Operator(Operators::SubEq);
                        remove(tokens, idx + 1);
                    }
                }
                Operators::Star => {
                    if let Tokens::Operator(Operators::Equal) = tokens[idx + 1] {
                        tokens[idx] = Tokens::Operator(Operators::MulEq);
                        remove(tokens, idx + 1);
                    }
                }
                Operators::Slash => {
                    if let Tokens::Operator(Operators::Equal) = tokens[idx + 1] {
                        tokens[idx] = Tokens::Operator(Operators::DivEq);
                        remove(tokens, idx + 1);
                    } else if let Tokens::Operator(Operators::Slash) = tokens[idx + 1] {
                        let mut i = idx + 1;
                        loop {
                            if let Tokens::Whitespace(str) = &tokens[i] {
                                if str == "\n" {
                                    break;
                                }
                            }
                            remove(tokens, i);
                            i += 1;
                        }
                        remove(tokens, idx);
                        return i - idx;
                    } else if let Tokens::Operator(Operators::Star) = tokens[idx + 1] {
                        let mut i = idx;
                        loop {
                            if let Tokens::Operator(Operators::Star) = &tokens[i] {
                                if let Tokens::Operator(Operators::Slash) = &tokens[i + 1] {
                                    break;
                                }
                            }
                            remove(tokens, i);
                            i += 1;
                        }
                        remove(tokens, i);
                        remove(tokens, i + 1);
                        return i - idx + 2;
                    }
                }
                Operators::Not => {
                    if let Tokens::Operator(Operators::Equal) = tokens[idx + 1] {
                        tokens[idx] = Tokens::Operator(Operators::NotEqual);
                        remove(tokens, idx + 1);
                    }
                }
                Operators::Equal => {
                    if let Tokens::Operator(Operators::Equal) = tokens[idx + 1] {
                        tokens[idx] = Tokens::Operator(Operators::DoubleEq);
                        remove(tokens, idx + 1);
                    }
                }
                Operators::Pipe => {
                    if let Tokens::Operator(Operators::Pipe) = tokens[idx + 1] {
                        tokens[idx] = Tokens::Operator(Operators::Or);
                        remove(tokens, idx + 1);
                    }
                }
                Operators::Ampersant => {
                    if let Tokens::Operator(Operators::Ampersant) = tokens[idx + 1] {
                        tokens[idx] = Tokens::Operator(Operators::And);
                        remove(tokens, idx + 1);
                    }
                }
                Operators::AngleBracket(bol) => {
                    if let Tokens::Operator(eq) = tokens[idx + 1] {
                        if let Operators::Equal = eq {
                            match *bol {
                                true => {
                                    tokens[idx] = Tokens::Operator(Operators::LessEq)
                                }
                                false => {
                                    tokens[idx] = Tokens::Operator(Operators::MoreEq)
                                }
                            }
                            remove(tokens, idx + 1);
                        }
                    }
                }
                _ => {}
            },
            _ => {
            }
        }
        1
    }
    fn clear(tokens: &Vec<Tokens>, lines: &Vec<(usize, usize)>) -> (Vec<Tokens>, Vec<(usize, usize)>) {
        let mut result = (Vec::new(), Vec::new());
        for (i, tok) in tokens.iter().enumerate() {
            if *tok != Tokens::Deleted {
                result.0.push((*tok).clone());
                result.1.push(lines[i].clone());
            }
        }
        result
    }
    fn not_end(idx: usize, tokens: &Vec<Tokens>) -> bool {
        idx < tokens.len()
    }
    fn remove(tokens: &mut Vec<Tokens>, idx: usize) -> usize {
        tokens[idx] = Tokens::Deleted;
        0
    }
    fn remove_range(tokens: &mut Vec<Tokens>, start: usize, end: usize) -> usize {
        for i in start..end {
            tokens[i] = Tokens::Deleted;
        }
        0
    }
    pub enum LexingErr {}
}

pub mod parse_err {
    use crate::tree_walker::tree_walker::Line;


    #[derive(Debug)]
    pub enum Errors {
        // number
        InvalidNumber(Line, String),
        // character
        InvalidChar(Line, String),
        // character
        CharacterTooLong(Line, String),
    }
}
