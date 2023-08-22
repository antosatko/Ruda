pub mod compiler_data {
}




















/*
match &this.nodes[cursor] {
    NodeType::Expect(tok) => {
        if let Tokens::Text(txt) = &tok.kind {
            
        }else {
            if token_cmp(tok.kind.to_owned(), tokens[*idx].to_owned()) {

            }else {
                return Err(AnalyzeErr::Expected(tok.kind.clone(), tokens[*idx].clone()))
            }
        }
    }
    NodeType::Maybe(tok) => {}
    NodeType::ArgsCondition(con) => {}
    NodeType::Command(comm) => {
        if let Tokens::Text(txt) = &comm.kind {
            match txt.as_str() {
                "end" => {
                    return Ok(result) ;
                }
                "err" => {
                    return Err(AnalyzeErr::Placeholder) ;
                }
                _ => {
                    println!("Unrecognized command: {}", &txt);
                }
            }
        }
    }
}
cursor += 1;*/