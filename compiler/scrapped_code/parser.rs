pub mod syntax {
    use crate::lexer::compiler_data::*;
    pub fn parse_sequence(kind: Kinds, tokens: &mut Vec<Tokens>) -> Result<Vec<Tokens>, ParseErr> {
        match kind {
            Kinds::Block => {
                let mut cur_brackets = 1;
                let mut i = 1;
                if let Tokens::CurlyBracket(bol) = tokens[0] {
                    if bol {
                        return Err(ParseErr::UnexpectedToken);
                        // syntax err: expected "{" at the start of code block, found "}"
                    }
                } else {
                    return Err(ParseErr::UnexpectedToken);
                    // syntax err: expected "{" at the start of code block
                }
                loop {
                    if i == tokens.len() {
                        return Err(ParseErr::FileEnd);
                        // syntax err: end of block never found
                    }
                    if let Tokens::CurlyBracket(bol) = tokens[i] {
                        cur_brackets += if bol { -1 } else { 1 }
                    }
                    if cur_brackets == 0 {
                        return Ok(tokens.drain(..i).collect());
                    }
                    i += 1;
                }
            }
            Kinds::Token(token) => {}
            Kinds::Value(end) => {
                let mut i = 0;
                let mut brackets = (0, 0, 0, 0);
                'main: loop {
                    if i == tokens.len() {
                        return Err(ParseErr::FileEnd);
                    }
                    match tokens[i] {
                        Tokens::Parenteses(bol) => {
                            if bol {
                                brackets.0 -= 1;
                                if brackets.0 == -1 {
                                    break;
                                }
                            } else {
                                for tok in &end {
                                    if let Tokens::Parenteses(boool) = tok {
                                        if brackets.0 == 0
                                            && brackets.1 == 0
                                            && brackets.2 == 0
                                            && brackets.3 == 0
                                            && *boool == bol
                                        {
                                            break 'main;
                                        }
                                    }
                                }
                                brackets.0 += 1;
                            }
                        }
                        Tokens::Operators(AngleBracket(bol) => {
                            if bol {
                                brackets.1 -= 1;
                                if brackets.1 == -1 {
                                    break;
                                }
                            } else {
                                for tok in &end {
                                    if let Tokens::Operators(AngleBracket(boool) = tok {
                                        if brackets.0 == 0
                                            && brackets.1 == 0
                                            && brackets.2 == 0
                                            && brackets.3 == 0
                                            && *boool == bol
                                        {
                                            break 'main;
                                        }
                                    }
                                }
                                brackets.1 += 1;
                            }
                        }
                        Tokens::CurlyBracket(bol) => {
                            if bol {
                                brackets.2 -= 1;
                                if brackets.2 == -1 {
                                    break;
                                }
                            } else {
                                for tok in &end {
                                    if let Tokens::CurlyBracket(boool) = tok {
                                        if brackets.0 == 0
                                            && brackets.1 == 0
                                            && brackets.2 == 0
                                            && brackets.3 == 0
                                            && *boool == bol
                                        {
                                            break 'main;
                                        }
                                    }
                                }
                                brackets.2 += 1;
                            }
                        }
                        Tokens::SquareBracket(bol) => {
                            if bol {
                                brackets.3 -= 1;
                                if brackets.3 == -1 {
                                    break;
                                }
                            } else {
                                for tok in &end {
                                    if let Tokens::SquareBracket(boool) = tok {
                                        if brackets.0 == 0
                                            && brackets.1 == 0
                                            && brackets.2 == 0
                                            && brackets.3 == 0
                                            && *boool == bol
                                        {
                                            break 'main;
                                        }
                                    }
                                }
                                brackets.3 += 1;
                            }
                        }
                        Tokens::Semicolon => {
                            for tok in &end {
                                if let Tokens::Semicolon = tok {
                                    if brackets.0 == 0
                                        && brackets.1 == 0
                                        && brackets.2 == 0
                                        && brackets.3 == 0
                                    {
                                        break 'main;
                                    }
                                }
                            }
                        }
                        Tokens::Colon => {
                            for tok in &end {
                                if let Tokens::Colon = tok {
                                    if brackets.0 == 0
                                        && brackets.1 == 0
                                        && brackets.2 == 0
                                        && brackets.3 == 0
                                    {
                                        break 'main;
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                    i += 1;
                }
                return Ok(tokens.drain(..i).collect());
            }
            Kinds::Word(txt) => {}
        }
        Err(ParseErr::None)
    }
    fn get_rules() -> Vec<SyntaxNodeHead> {
        vec![
            SyntaxNodeHead {
                kind: Tokens::Keyword(Keywords::If),
                node: vec![
                    NodeOp::Expect(SyntaxNode {
                        kind: Kinds::Value(vec![Tokens::CurlyBracket(false), Tokens::Colon]),
                        node: vec![],
                    }),
                    NodeOp::Expect(SyntaxNode {
                        kind: Kinds::Block,
                        node: vec![],
                    }),
                    NodeOp::Maybe(SyntaxNode {
                        kind: Kinds::Word(String::from("else if")),
                        node: vec![
                            NodeOp::Expect(SyntaxNode {
                                kind: Kinds::Value(vec![
                                    Tokens::CurlyBracket(false),
                                    Tokens::Colon,
                                ]),
                                node: vec![],
                            }),
                            NodeOp::Expect(SyntaxNode {
                                kind: Kinds::Block,
                                node: vec![],
                            }),
                            NodeOp::Jmp(3),
                        ],
                    }),
                    NodeOp::Maybe(SyntaxNode {
                        kind: Kinds::Word(String::from("else")),
                        node: vec![NodeOp::Expect(SyntaxNode {
                            kind: Kinds::Block,
                            node: vec![],
                        })],
                    }),
                ],
            },
            SyntaxNodeHead {
                kind: Tokens::Keyword(Keywords::Switch),
                node: vec![
                    NodeOp::Expect(SyntaxNode {
                        kind: Kinds::Word(String::from("{")),
                        node: vec![
                            NodeOp::Maybe(SyntaxNode {
                                kind: Kinds::Value(vec![Tokens::CurlyBracket(false)]),
                                node: vec![
                                    NodeOp::Expect(SyntaxNode {
                                        kind: Kinds::Block,
                                        node: vec![],
                                    }),
                                    NodeOp::Jmp(2),
                                ],
                            }),
                            NodeOp::Maybe(SyntaxNode {
                                kind: Kinds::Word(String::from("_")),
                                node: vec![NodeOp::Expect(SyntaxNode {
                                    kind: Kinds::Block,
                                    node: vec![],
                                })],
                            }),
                            NodeOp::Jmp(3),
                        ],
                    }),
                    NodeOp::Expect(SyntaxNode {
                        kind: Kinds::Word(String::from("}")),
                        node: vec![],
                    }),
                ],
            },
        ]
    }
    pub struct SyntaxNode {
        kind: Kinds,
        node: Vec<NodeOp>,
    }
    pub struct SyntaxNodeHead {
        kind: Tokens,
        node: Vec<NodeOp>,
    }
    pub enum NodeOp {
        Expect(SyntaxNode),
        Maybe(SyntaxNode),
        Jmp(i128),
        End(Tokens),
    }
    #[derive(PartialEq, Eq)]
    pub enum Kinds {
        Token(Tokens),
        Block,
        Value(Vec<Tokens>),
        Word(String),
    }
    #[derive(Debug)]
    pub enum ParseErr {
        None,
        UnexpectedToken,
        FileEnd,
    }
}
