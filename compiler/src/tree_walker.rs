pub mod tree_walker {
    use std::collections::HashMap;

    use crate::ast_parser::ast_parser::{self, *};
    use crate::lexer::tokenizer::{*, self};
    pub fn generate_tree(tokens: &Vec<Tokens>, syntax: (&Tree, &mut Vec<HeadParam>), lines: &Vec<(usize, usize)>) -> Option<(Node, HashMap<String, ArgNodeType>)> {
        let mut idx = 0;
        let mut globals_data = HashMap::new();
        for global in syntax.1 {
            match global {
                HeadParam::Array(arr) => {
                    globals_data.insert(arr.to_string(), ArgNodeType::Array(vec![]));
                }
                HeadParam::Value(val) => {
                    globals_data.insert(
                        val.to_string(),
                        ArgNodeType::Value(Node {
                            name: Tokens::Text(String::from("'none")),
                            data: None,
                            nodes: HashMap::new(),
                        }),
                    );
                }
            }
        }
        let product = parse_node(
            &tokens,
            &syntax.0,
            &HashMap::new(),
            &mut idx,
            &String::from("entry"),
            &mut globals_data
        );
        match product {
            Ok(prd) => {
                Some((prd, globals_data))
            }
            Err(err) => {
                println!("{err:?}\nOriginated at line: {}, column: {}", lines[idx].1 + 1, lines[idx].0 + 1);
                None
            }
        }
    }
    fn prep_nodes(syntax: &Tree, id: &String) -> Option<HashMap<String, ArgNodeType>> {
        let mut map = HashMap::new();
        match &syntax.get(id) {
            Some(node) => {
                for param in &node.parameters {
                    match param {
                        HeadParam::Array(arr) => {
                            map.insert(arr.into(), ArgNodeType::Array(vec![]));
                        }
                        HeadParam::Value(val) => {
                            map.insert(
                                val.into(),
                                ArgNodeType::Value(Node {
                                    name: Tokens::Text(String::from("'none")),
                                    data: None,
                                    nodes: HashMap::new(),
                                }),
                            );
                        }
                    }
                }
            }
            None => {
                return None;
            }
        }
        Some(map)
    }
    /// TODO: make standalone recursive scope function
    pub fn parse_node(
        tokens: &Vec<Tokens>,
        syntax: &Tree,
        params: &NodeParameters,
        idx: &mut usize,
        id: &String,
        globals: &mut HashMap<String, ArgNodeType>
    ) -> Result<Node, (Err, bool)> {
        let mut result = Node {
            name: Tokens::Text(id.into()),
            data: None,
            nodes: prep_nodes(&syntax, &id).expect(&format!("{id}")),
        };
        match parse_scope(
            &tokens,
            &syntax,
            &params,
            idx,
            &syntax.get(id).unwrap().nodes,
            &mut result.nodes,
            &mut false,
            globals
        ) {
            Ok(_) => {
                return Ok(result);
            }
            Err(err) => {
                return Err(err);
            }
        }
    }
    /// returns how many lines should prev. scope go back or Err
    fn parse_scope(
        tokens: &Vec<Tokens>,
        syntax: &Tree,
        params: &NodeParameters,
        idx: &mut usize,
        nodes: &Vec<ast_parser::NodeType>,
        data: &mut HashMap<String, super::tree_walker::ArgNodeType>,
        harderr: &mut bool,
        globals: &mut HashMap<String, ArgNodeType>
    ) -> Result<Option<(usize, ReturnActions)>, (Err, bool)> {
        let mut node_idx = 0;
        let mut advance_tok;
        let mut advance_node;

        macro_rules! Advance {
            () => {
                if advance_node {
                    node_idx += 1;
                }
                if advance_tok {
                    *idx += 1;
                }
            };
        }
        macro_rules! Back {
            ($num: expr, $freeze: expr) => {
                //println!("{}   {}", $num, node_idx);
                if $num <= node_idx {
                    if $freeze == ReturnActions::Freeze {
                        advance_node = false;
                    }
                    node_idx -= $num;
                } else {
                    //println!("Warpin away!\n\n\n");
                    *idx += 1;
                    return Ok(Some(($num - node_idx - 1, $freeze)));
                }
            };
        }
        macro_rules! ArgsCheck {
            ($args: expr, $node: expr, $token: expr) => {
                if let Some(arg) = $args.get("print") {
                    println!("{:?}", arg);
                }
                if let Some(_) = $args.get("debug") {
                    println!("{:?}", tokens[*idx]);
                }
                if let Some(_) = $args.get("peek") {
                    advance_tok = false;
                }
                if let Some(num) = $args.get("advance") {
                    match num.parse::<usize>() {
                        Ok(num) => {
                            *idx += num;
                        }
                        _ => {
                            *idx += 1;
                        }
                    }
                }
                if let Some(num) = $args.get("#advance") {
                    match num.parse::<usize>() {
                        Ok(num) => {
                            *idx -= num;
                        }
                        _ => {
                            *idx -= 1;
                        }
                    }
                }
                if let Some(arg) = $args.get("harderr") {
                    *harderr = false;
                    if arg == "true" {
                        *harderr = true;
                    }
                }
                if let Some(arg) = $args.get("notempty") {
                    if let TokenOrNode::Node(ref nodede) = $token {
                        if let Some(ArgNodeType::Array(arr)) = nodede.nodes.get(arg) {
                            if arr.len() == 0 {
                                Error!(Err::ExpectedOneOf(extract_tokens_range(&nodes, (maybes_start(&nodes, node_idx - 1), maybes_end(&nodes, node_idx))), tokens[*idx].clone()), true);
                            }
                        }
                    }
                }
                else if let Some(arg) = $args.get("set") {
                    // Split the argument by space
                    for arg_part in arg.split_whitespace() {
                        // Find place of arg_part
                        let mut place = match data.get_mut(arg_part.into()) {
                            Some(place) => place,
                            None => {
                                if let Some(place) = globals.get_mut(arg_part.into()) {
                                    place
                                } else {
                                    Error!(Err::EmptyNodeParameter(arg_part.into()), true);
                                    continue; // Skip to the next part of the argument
                                }
                            }
                        };
                        set($token, $node, &mut place);
                    }
                }
                if let Some(ident) = $args.get("data") {
                    println!("{:?}", data.get(ident).expect("wrong identifier for data"));
                }
                if let Some(arg) = $args.get("back") {
                    let num = arg.parse::<usize>().unwrap();
                    Back!(num, ReturnActions::Freeze);
                }
                if let Some(str) = $args.get("end") {
                    if str == "true" {
                        *idx += 1;
                    }
                    break;
                }
                if let Some(arg) = $args.get("pass") {
                    Error!(Err::Pass(arg.into()), false);
                }
                if let Some(arg) = $args.get("err") {
                    if arg.len() > 0 {
                        Error!(Err::Msg(arg.into()), false);
                    }
                    Error!($node);
                }
            };
        }
        macro_rules! ScopeEnter {
            ($node: expr, $freeze: expr) => {
                match parse_scope(&tokens, &syntax, &params, idx, &$node.nodes, data, harderr, globals) {
                    Ok(back) => match back {
                        Some(back) => match back.1 {
                            ReturnActions::Freeze => {
                                Back!(back.0, back.1);
                            }
                            ReturnActions::Nothing => {
                                Back!(back.0, back.1);
                            }
                            ReturnActions::Chain => return Ok(Some(back)),
                            ReturnActions::Advance => {
                                advance_tok = false;
                            }
                        },
                        _ => {}
                    },
                    Err(err) => {
                        return Err(err);
                    }
                }
            };
        }
        macro_rules! OpenStruct {
            ($ident: expr, $node: expr, $recoverable: expr) => {
                let start_idx = *idx;
                match parse_node(&tokens, &syntax, &$node.arguments, idx, &$ident.into(), globals) {
                    Ok(nd) => {
                        ScopeEnter!(&$node, true);
                        ArgsCheck!(&$node.arguments, &$node.kind, TokenOrNode::Node(nd.clone()));
                        advance_tok = false;
                        Advance!();
                    }
                    Err(err) => match err.0 {
                        /*Err::Pass(arg) => {
                            let tok = 'look_for_stop: loop {
                                node_idx += 1;
                                if node_idx >= nodes.len() {
                                    Error!(
                                        Err::Msg(String::from(
                                            "Could not find break with matching name"
                                        )),
                                        true
                                    );
                                }
                                if match &nodes[node_idx] {
                                    NodeType::Expect(node) => {
                                        node.arguments.get("stop") == Some(&arg)
                                    }
                                    NodeType::Maybe(node) => {
                                        node.arguments.get("stop") == Some(&arg)
                                    }
                                    _ => false,
                                } {
                                    break 'look_for_stop;
                                }
                            };
                        }*/
                        Err::FileEndPeaceful => return Err((Err::FileEndPeaceful, false)),
                        _ => {
                            /*println!(
                                "BYL JSEM TU {} {} {} = {}",
                                err.1,
                                $recoverable,
                                harderr,
                                err.1 || !$recoverable
                            );*/
                            advance_tok = false;
                            if err.1 || !$recoverable {
                                Error!(err.0, !$recoverable);
                            }
                            *idx = start_idx;
                            advance_node = false;
                            if !err.1 {
                                advance_node = true;
                            }
                            Advance!();
                        }
                    },
                }
            };
        }
        macro_rules! Error {
            ($error: expr, $reset: expr) => {
                //println!("Erorruju: {:?}", $error);
                return Err(($error, *harderr));
            };
            ($node: expr) => {
                if node_idx == 0 {
                    Error!(Err::Expected($node.clone(), tokens[*idx].clone()), true);
                }
                Error!(Err::ExpectedOneOf(extract_tokens_range(&nodes, (maybes_start(&nodes, node_idx - 1), maybes_end(&nodes, node_idx))), tokens[*idx].clone()), true);
            }
        }
        let endon = if let Some(arg) = params.get("endon") {
            Some(tokenizer::parse_token(arg))
        }else {
            None
        };
        let endwith = if let Some(arg) = params.get("endwith") {
            Some(tokenizer::parse_token(arg))
        }else {
            None
        };
        advance_tok = false;
        while node_idx < nodes.len() {
            advance_node = true;
            advance_tok = true;
            //println!("nodes: {:?} idx: {node_idx}\ntokens: {:?} idx: {}", nodes[node_idx], tokens[*idx], *idx);
            if *idx >= tokens.len() {
                if maybes_end(nodes, node_idx) == nodes.len() {
                    //println!("Advancin'");
                    return Ok(Some((0, ReturnActions::Advance)));
                }
                //println!("nodes: {} idx: {node_idx}\ntokens: {} idx: {}", nodes.len(), tokens.len(), *idx);
                if node_idx >= nodes.len() - 1 {
                    //println!("peaceful end");
                    return Err((Err::FileEndPeaceful, true));
                }
                //println!("KONEC SOUBORU PREJ");
                Error!(Err::FileEnd, true);
            }
            if let Some(ref endontok) = endon {
                if tokens[*idx] == *endontok {
                    return Ok(Some((0, ReturnActions::Chain)));
                }
            }
            if let Some(ref endontok) = endwith {
                if tokens[*idx] == *endontok {
                    *idx += 1;
                    return Ok(Some((0, ReturnActions::Chain)));
                }
            }
            match &nodes[node_idx] {
                NodeType::Expect(node) => {
                    //println!("{:?}    {:?}", node.kind, tokens[*idx]);
                    match token_cmp(&node.kind, &tokens[*idx]) {
                        CompareResult::Eq => {
                            // match
                            ArgsCheck!(
                                &node.arguments,
                                &node.kind,
                                TokenOrNode::Token(tokens[*idx].clone())
                            );
                            Advance!();
                        }
                        CompareResult::NotEq => {
                            // err
                            Error!(node.kind);
                        }
                        CompareResult::Ident(ident) => {
                            OpenStruct!(ident, &node, false);
                        }
                    }
                }
                NodeType::Maybe(node) => {
                    //println!("{:?}    {:?}", node.kind, tokens[*idx]);
                    match token_cmp(&node.kind, &tokens[*idx]) {
                        CompareResult::Eq => {
                            // match
                            ArgsCheck!(
                                &node.arguments,
                                &node.kind,
                                TokenOrNode::Token(tokens[*idx].clone())
                            );
                            Advance!();
                            ScopeEnter!(node, false);
                        }
                        CompareResult::NotEq => {
                            // err
                            //if node_idx == maybes_end(nodes, node_idx) - 1 {
                            advance_tok = false;
                            //}
                            Advance!();
                        }
                        CompareResult::Ident(ident) => {
                            OpenStruct!(ident, &node, true);
                        }
                    }
                }
                NodeType::Command(node) => {
                    if let Tokens::Text(str) = &node.kind {
                        //println!("Command: {}", str);
                        match str.as_str() {
                            "end" => {
                                //println!("\n\n\n\n\n\n\n\n\n\n\n");
                                return Ok(Some((0, ReturnActions::Chain)));
                            }
                            "notempty" => {
                                match node.arguments.get("nodes") {
                                    Some(args) => {
                                        for arg in args.split(" "){
                                            if let Some(ArgNodeType::Array(arr)) = data.get(arg) {
                                                if arr.len() == 0 {
                                                    *harderr = true;
                                                    Error!(node.kind);
                                                }
                                            }
                                        }
                                    }
                                    None => {
                                        panic!("parameter doesnt exist.");
                                    }
                                } 
                            }
                            _ => {}
                        }
                    }
                    advance_tok = false;
                    Advance!();
                }
                NodeType::ArgsCondition(args_con) => {
                    *idx -= 1;
                    let mut all_match = true;
                    for arg in &args_con.params {
                        //println!("zkouska siren: {arg:?}");
                        match params.get(arg.0) {
                            Some(param) => {
                                if param != arg.1 {
                                //println!("problem");
                                all_match = false;
                                }
                            }
                            None => {
                                //println!("problem");
                                all_match = false;
                            }
                        }
                    }
                    if all_match {
                        advance_tok = false;
                        Advance!();
                        ScopeEnter!(args_con, true);
                        advance_tok = false;
                        advance_node = false;
                        Advance!();
                    }
                }
            }
        }
        let advnc = if advance_tok {1usize} else {0};
        if let Some(ref endontok) = endwith {
            if tokens.len() > *idx && tokens[*idx - advnc] == *endontok {
                //println!("tokens we ended with: {:?}", tokens[*idx]);
                *idx += 1;
                //println!("tokens we ended with: {:?}", tokens[*idx]);
                return Ok(Some((0, ReturnActions::Chain)));
            }else {
                //println!("{:?} {:?} {:?}", params, tokens[*idx], *endontok);
                Error!(Err::WrongEndingToken((*endontok).clone(), tokens[*idx].clone()), true);
            }
        }
        if let Some(ref endontok) = endon {
            if tokens.len() > *idx && tokens[*idx - advnc] == *endontok {
                return Ok(Some((0, ReturnActions::Chain)));
            }else {
                //println!("{:?} {:?} {:?}", params, tokens[*idx], *endontok);
                Error!(Err::WrongEndingToken((*endontok).clone(), tokens[*idx].clone()), true);
            }
        }
        //println!("SsdDFWSFE");
        Ok(Some((0, ReturnActions::Advance)))
    }
    fn set(token_found: TokenOrNode, token_expected: &Tokens, place: &mut ArgNodeType) {
        match place {
            ArgNodeType::Array(arr) => match token_found {
                TokenOrNode::Node(node) => arr.push(node),
                TokenOrNode::Token(token) => arr.push(construct_token(&token, token_expected)),
            },
            ArgNodeType::Value(val) => match token_found {
                TokenOrNode::Node(node) => *val = node,
                TokenOrNode::Token(token) => *val = construct_token(&token, token_expected),
            },
        }
    }
    enum TokenOrNode {
        Token(Tokens),
        Node(Node),
    }
    fn construct_token(token_found: &Tokens, token_expected: &Tokens) -> Node {
        match token_expected {
            Tokens::String(_) => Node {
                name: token_found.clone(),
                data: Some(token_expected.clone()),
                nodes: HashMap::new(),
            },
            _ => Node {
                name: token_found.clone(),
                data: None,
                nodes: HashMap::new(),
            },
        }
    }
    fn token_cmp<'a>(tree_element: &'a Tokens, source_token: &'a Tokens) -> CompareResult<'a> {
        match tree_element {
            Tokens::String(ref txt) => match txt.as_str() {
                "'string" => {
                    if let Tokens::String(_) = source_token {
                        return CompareResult::Eq;
                    }
                    return CompareResult::NotEq;
                }
                "'number" => {
                    if let Tokens::Number(_, _) = source_token {
                        return CompareResult::Eq;
                    }
                    return CompareResult::NotEq;
                }
                "'float" => {
                    if let Tokens::Number(_, kind) = source_token {
                        return if *kind == 'f' {CompareResult::Eq} else {CompareResult::NotEq};
                    }
                    return CompareResult::NotEq;
                }
                "'int" => {
                    if let Tokens::Number(_, kind) = source_token {
                        return if *kind == 'i' {CompareResult::Eq} else {CompareResult::NotEq};
                    }
                    return CompareResult::NotEq;
                }
                "'text" => {
                    if let Tokens::Text(_) = source_token {
                        return CompareResult::Eq;
                    }
                    return CompareResult::NotEq;
                }
                "'char" => {
                    if let Tokens::Char(_) = source_token {
                        return CompareResult::Eq;
                    }
                    return CompareResult::NotEq;
                }
                "'eof" => {
                    if let Tokens::EndOfFile = source_token {
                        return CompareResult::Eq;
                    }
                    return CompareResult::NotEq;
                }
                "'any" => CompareResult::Eq,
                _ => {
                    if let Tokens::Text(str) = source_token {
                        if str == txt {
                            return CompareResult::Eq;
                        }
                        return CompareResult::NotEq;
                    }
                    return CompareResult::NotEq;
                }
            },
            Tokens::Text(ident) => {
                return CompareResult::Ident(&ident);
            }
            _ => {
                if *tree_element == *source_token {
                    return CompareResult::Eq;
                }
                return CompareResult::NotEq;
            }
        }
    }
    /// return end index+1 of maybes row
    fn maybes_end(syntax: &Vec<ast_parser::NodeType>, mut idx: usize) -> usize {
        while let ast_parser::NodeType::Maybe(_) = syntax[idx] {
            idx += 1;
            if idx == syntax.len() {
                break;
            }
        }
        idx
    }
    fn maybes_start(syntax: &Vec<ast_parser::NodeType>, mut idx: usize) -> usize {
        while let ast_parser::NodeType::Maybe(_) = syntax[idx] {
            if idx == 0 {
                break;
            }
            idx -= 1;
        }
        match syntax[idx] {
            NodeType::Expect(_) => idx,
            NodeType::Maybe(_) => idx,
            _=> idx + 1
        }
    }
    fn extract_tokens_range(syntax: &Vec<ast_parser::NodeType>, range: (usize, usize)) -> Vec<Tokens> {
        let mut result = Vec::new();
        for idx in range.0..range.1 {
            match &syntax[idx] {
                NodeType::Expect(tok) => {
                    result.push(tok.kind.clone());
                }
                NodeType::Maybe(tok) => {
                    result.push(tok.kind.clone());
                }
                _ => {
                }
            }
        }
        result
    }
    #[derive(PartialEq)]
    enum ReturnActions {
        Freeze,
        Nothing,
        Chain,
        Advance,
    }
    #[derive(PartialEq)]
    enum CompareResult<'a> {
        Eq,
        NotEq,
        Ident(&'a str),
    }
    #[derive(Debug)]
    pub enum Err {
        Expected(Tokens, Tokens),
        ExpectedOneOf(Vec<Tokens>, Tokens),
        Msg(String),
        FileEnd,
        FileEndPeaceful,
        Pass(String),
        /// expected found
        WrongEndingToken(Tokens, Tokens),
        EmptyNodeParameter(String),
    }
    /// structures defined by user
    #[derive(Debug, Clone)]
    pub struct Node {
        pub name: Tokens,
        pub data: Option<Tokens>,
        pub nodes: HashMap<String, ArgNodeType>,
    }
    #[derive(Debug, Clone)]
    pub enum ArgNodeType {
        Array(Vec<Node>),
        Value(Node),
    }
    impl ArgNodeType {
        pub fn get_value(&self) -> &Node {
            if let Self::Value(val) = self {
                return val
            }else {
                panic!()
            }
        }
        pub fn get_array(&self) -> &Vec<Node> {
            if let Self::Array(val) = self {
                return val
            }else {
                panic!()
            }
        }
    }
}