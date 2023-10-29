pub mod ast_parser {
    pub type Tree = HashMap<String, Head>;
    pub type NodeParameters = HashMap<String, String>;
    use std::collections::HashMap;

    use super::formater::refactor;
    use crate::lexer::tokenizer::*;
    pub fn generate_ast(source_path: &str) -> Option<(Tree, Vec<HeadParam>)> {
        use std::fs;
        let source = match fs::read_to_string(source_path) {
            Ok(source) => source,
            Err(_) => {
                return None;
            }
        };
        let (tokens, mut lines, mut errors) = tokenize(&source.as_bytes(), false);
        if let Ok(mut refactored) = refactor(tokens, &mut lines, &mut errors) {
            return Some(analize_tree(&mut refactored));
        } else {
            println!(
                "Could not parse AST {source_path}, number of errors: {}",
                errors.len()
            );
            return None;
        }
    }
    fn analize_tree(tokens: &mut Vec<Tokens>) -> (Tree, Vec<HeadParam>) {
        let mut hash_map = HashMap::new();
        let mut idx = 0;
        while let Some(head) = read_head(tokens, &mut idx) {
            hash_map.insert(head.name.to_string(), head);
            idx += 1;
        }
        let globals = get_globals(&hash_map);
        (hash_map, globals)
    }
    fn get_globals(tree: &HashMap<String, Head>) -> Vec<HeadParam> {
        let mut globals = vec![];
        for (name, node) in tree {
            if name == "globals" {
                for param in &node.parameters {
                    let global = match param {
                        HeadParam::Array(name) => HeadParam::Array(name.to_string()),
                        HeadParam::Value(name) => HeadParam::Value(name.to_string()),
                    };
                    globals.push(global);
                }
            }
        }
        globals
    }
    fn read_head(tokens: &mut Vec<Tokens>, idx: &mut usize) -> Option<Head> {
        if tokens.len() == *idx {
            return None;
        }
        let name = loop {
            if let Tokens::Text(txt) = &tokens[*idx] {
                *idx += 1;
                break txt.to_string();
            }
            *idx += 1;
        };
        let mut parameters = Vec::with_capacity(tokens.len() / 100);
        while tokens[*idx] != Tokens::Tab {
            match &tokens[*idx] {
                Tokens::SquareBracket(closed) => {
                    if !closed {
                        if let Tokens::Text(txt) = &tokens[*idx + 1] {
                            parameters.push(HeadParam::Array(txt.to_string()));
                            *idx += 2;
                        } else {
                            return None;
                        }
                    } else {
                        return None;
                    }
                }
                Tokens::Text(txt) => {
                    parameters.push(HeadParam::Value(txt.to_string()));
                    *idx += 1;
                }
                _ => {}
            }
        }
        Some(Head {
            name,
            parameters,
            nodes: get_nodes(tokens, idx, 1),
        })
    }
    fn get_nodes(tokens: &mut Vec<Tokens>, idx: &mut usize, tabs: usize) -> Vec<NodeType> {
        let mut nodes: Vec<NodeType> = vec![];
        while count_tabs(&tokens, *idx) == tabs {
            *idx += tabs;
            match tokens[*idx + 1] {
                Tokens::Optional => {
                    tokens.remove(*idx + 1);
                    let node = NodeType::Maybe(Node {
                        kind: tokens.remove(*idx),
                        arguments: get_node_args(&tokens, idx),
                        nodes: get_nodes(tokens, idx, tabs + 1),
                    });
                    nodes.push(node);
                }
                Tokens::Operator(op) => match op {
                    Operators::Not => {
                        tokens.remove(*idx + 1);
                        let node = NodeType::Command(Node {
                            kind: tokens.remove(*idx),
                            arguments: get_node_args(&tokens, idx),
                            nodes: get_nodes(tokens, idx, tabs + 1),
                        });
                        nodes.push(node);
                    }
                    Operators::Equal => {
                        let node = NodeType::ArgsCondition(ArgsCon {
                            params: get_node_args(&tokens, idx),
                            nodes: get_nodes(tokens, idx, tabs + 1),
                        });
                        nodes.push(node);
                    }
                    _ => {}
                },
                _ => {
                    let node = NodeType::Expect(Node {
                        kind: tokens.remove(*idx),
                        arguments: get_node_args(&tokens, idx),
                        nodes: get_nodes(tokens, idx, tabs + 1),
                    });
                    nodes.push(node);
                }
            }
        }
        nodes
    }
    fn get_node_args(tokens: &Vec<Tokens>, idx: &mut usize) -> NodeParameters {
        let mut args = HashMap::new();
        while let Tokens::Text(name) = &tokens[*idx] {
            *idx += 2;
            if let Tokens::String(value) = &tokens[*idx] {
                args.insert(name.to_string(), value.to_string());
            }
            *idx += 1;
        }
        args
    }
    fn count_tabs(tokens: &Vec<Tokens>, idx: usize) -> usize {
        let mut count = 0;
        while let Tokens::Tab = &tokens[idx + count] {
            count += 1;
        }
        count
    }
    #[derive(Debug)]
    pub struct Head {
        pub name: String,
        pub parameters: Vec<HeadParam>,
        pub nodes: Vec<NodeType>,
    }
    #[derive(Debug)]
    pub enum HeadParam {
        Array(String),
        Value(String),
    }
    #[derive(Debug)]
    pub struct Node {
        pub kind: Tokens,
        pub arguments: NodeParameters,
        pub nodes: Vec<NodeType>,
    }
    #[derive(Debug)]
    pub enum NodeType {
        Maybe(Node),
        Expect(Node),
        Command(Node),
        ArgsCondition(ArgsCon),
    }
    #[derive(Debug)]
    pub struct ArgsCon {
        pub params: NodeParameters,
        pub nodes: Vec<NodeType>,
    }
}

mod formater {
    use crate::{
        lexer::tokenizer::{/*Keywords,*/ deparse_token, Operators, Tokens},
        lexing_preprocessor::{lexing_preprocessor::LexingErr, parse_err::Errors},
        tree_walker::tree_walker::Line,
    };

    pub fn refactor(
        mut tokens: Vec<Tokens>,
        lines: &mut Vec<(usize, usize)>,
        errors: &mut Vec<Errors>,
    ) -> Result<Vec<Tokens>, LexingErr> {
        let mut i = 0;
        while i < tokens.len() {
            i += process_token(&mut tokens, i, lines, errors);
        }
        Ok(tokens)
    }
    fn process_token(
        tokens: &mut Vec<Tokens>,
        idx: usize,
        lines: &mut Vec<(usize, usize)>,
        errors: &mut Vec<Errors>,
    ) -> usize {
        match &tokens[idx] {
            Tokens::DoubleQuotes => {
                let mut i = idx + 1;
                let mut res = String::new();
                while tokens[i] != Tokens::DoubleQuotes {
                    res.push_str(&deparse_token(&tokens[i]));
                    i += 1;
                    if i == tokens.len() {
                        // syntax err: end of string never found
                        tokens.splice(idx + 1.., []);
                        lines.splice(idx + 1.., []);
                        tokens[idx] = Tokens::String(res);
                        return 1;
                    }
                }
                tokens.splice(idx + 1..i + 1, []);
                lines.splice(idx + 1..i + 1, []);
                tokens[idx] = Tokens::String(res);
            }
            Tokens::Space => {
                tokens.remove(idx);
                lines.remove(idx);
                return 0;
            }
            Tokens::Colon => {
                if let Tokens::Colon = tokens[idx + 1] {
                    tokens[idx] = Tokens::DoubleColon;
                    tokens.remove(idx + 1);
                    lines.remove(idx + 1);
                }
            }
            Tokens::Text(txt) => {
                let bytes = txt.as_bytes();
                if let Some(first) = bytes.get(0) {
                    if first.is_ascii_digit() {
                        // float
                        if let Tokens::Dot = &tokens[idx + 1] {
                            let first_num = if let Ok(num) = txt.parse::<usize>() {
                                num
                            } else {
                                // syntax err: incorrect number
                                errors.push(Errors::InvalidNumber(
                                    Line::from(lines[idx]),
                                    txt.to_string(),
                                ));
                                return 1;
                            };
                            if let Tokens::Text(txt2) = &tokens[idx + 2] {
                                let mut float = String::from("0.");
                                float.push_str(txt2);
                                if let Ok(num2) = float.parse::<f64>() {
                                    tokens[idx] = Tokens::Number(first_num as f64 + num2, 'f');
                                    tokens.remove(idx + 1);
                                    tokens.remove(idx + 1);
                                    lines.remove(idx + 1);
                                    lines.remove(idx + 1);
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
                            if bytes[bytes.len() - 1].is_ascii_digit() {
                                if let Ok(num) = txt.parse::<usize>() {
                                    tokens[idx] = Tokens::Number(num as f64, 'i')
                                } else {
                                    errors.push(Errors::InvalidNumber(
                                        Line::from(lines[idx]),
                                        txt.to_string(),
                                    ));
                                    // syntax err: incorrect number
                                }
                            } else {
                                if let Ok(num) = txt[..txt.len() - 1].parse::<usize>() {
                                    tokens[idx] =
                                        Tokens::Number(num as f64, bytes[bytes.len() - 1] as char)
                                } else {
                                    errors.push(Errors::InvalidNumber(
                                        Line::from(lines[idx]),
                                        txt.to_string(),
                                    ));
                                    // syntax err: incorrect number
                                }
                            }
                        }
                        return 1;
                    }
                }
                // nesting
                for char in txt.chars() {
                    if !char.is_whitespace() {
                        return 1;
                    }
                }
                lines.remove(idx);
                tokens.remove(idx);
                return 0;
            }
            Tokens::Whitespace(txt) => {
                if txt == "\t" {
                    tokens[idx] = Tokens::Tab;
                    return 1;
                }
                lines.remove(idx);
                tokens.remove(idx);
                return 0;
            }
            Tokens::Operator(op) => match op {
                Operators::Plus => {
                    if let Tokens::Operator(Operators::Equal) = tokens[idx + 1] {
                        tokens[idx] = Tokens::Operator(Operators::AddEq);
                        tokens.remove(idx + 1);
                        lines.remove(idx + 1);
                    }
                }
                Operators::Minus => {
                    if let Tokens::Operator(Operators::Equal) = tokens[idx + 1] {
                        tokens[idx] = Tokens::Operator(Operators::SubEq);
                        tokens.remove(idx + 1);
                        lines.remove(idx + 1);
                    }
                }
                Operators::Star => {
                    if let Tokens::Operator(Operators::Equal) = tokens[idx + 1] {
                        tokens[idx] = Tokens::Operator(Operators::MulEq);
                        tokens.remove(idx + 1);
                        lines.remove(idx + 1);
                    }
                }
                Operators::Slash => {
                    if let Tokens::Operator(Operators::Equal) = tokens[idx + 1] {
                        tokens[idx] = Tokens::Operator(Operators::DivEq);
                        tokens.remove(idx + 1);
                        lines.remove(idx + 1);
                    }
                }
                Operators::Not => {
                    if let Tokens::Operator(Operators::Equal) = tokens[idx + 1] {
                        tokens[idx] = Tokens::Operator(Operators::NotEqual);
                        tokens.remove(idx + 1);
                        lines.remove(idx + 1);
                    }
                }
                Operators::Equal => {
                    if let Tokens::Operator(Operators::Equal) = tokens[idx + 1] {
                        tokens[idx] = Tokens::Operator(Operators::DoubleEq);
                        tokens.remove(idx + 1);
                        lines.remove(idx + 1);
                    }
                }
                Operators::Pipe => {
                    if let Tokens::Operator(Operators::Pipe) = tokens[idx + 1] {
                        tokens[idx] = Tokens::Operator(Operators::Or);
                        tokens.remove(idx + 1);
                        lines.remove(idx + 1);
                    }
                }
                Operators::Ampersant => {
                    if let Tokens::Operator(Operators::Ampersant) = tokens[idx + 1] {
                        tokens[idx] = Tokens::Operator(Operators::And);
                        tokens.remove(idx + 1);
                        lines.remove(idx + 1);
                    }
                }
                Operators::AngleBracket(bol) => {
                    if let Tokens::Operator(eq) = tokens[idx + 1] {
                        if let Operators::Equal = eq {
                            tokens[idx] = match *bol {
                                true => Tokens::Operator(Operators::LessEq),
                                false => Tokens::Operator(Operators::MoreEq),
                            };
                            tokens.remove(idx + 1);
                            lines.remove(idx + 1);
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
        1
    }
}
