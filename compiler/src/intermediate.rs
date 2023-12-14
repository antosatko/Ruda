pub mod dictionary {
    use runtime::runtime_types;

    use super::{
        AnalyzationError::{self, ErrType},
        Kind, TypeBody,
    };
    use crate::{
        codeblock_parser,
        codegen::{self, InnerPath},
        expression_parser::{self, get_args, try_get_literal, Root, ValueType},
        lexer::tokenizer::{Operators, Tokens},
        libloader::{self, Const},
        tree_walker::tree_walker::{self, ArgNodeType, Err, Line, Node},
    };
    use core::panic;
    use std::{borrow::BorrowMut, collections::HashMap, fs::DirEntry, io::Read};

    pub fn from_ast(
        ast: &HashMap<String, tree_walker::ArgNodeType>,
        globals: &Vec<String>,
        file_name: &str,
    ) -> (Dictionary, Vec<ErrType>) {
        let mut global_dict = Dictionary::new();
        let mut errors = Vec::new();
        if let Some(ArgNodeType::Array(entry)) = ast.get("nodes") {
            load_dictionary(entry, &mut global_dict, &mut errors, file_name);
        }
        /*analyze_consts(&mut global_dict, &mut errors); // TODO: add this back
        println!("errors: {errors:?}");*/
        (global_dict, errors)
    }
    pub fn analyze_consts(dictionary: &mut Dictionary, errors: &mut Vec<ErrType>) {
        let mut changes = Vec::new();
        loop {
            for i in 0..dictionary.constants.len() {
                let constant = &dictionary.constants[i];
                if constant.real_value.is_some() {
                    continue;
                }
                if let Some(val) = analyze_const(&constant.value, dictionary, errors) {
                    changes.push((i, val));
                }
            }
            if changes.len() > 0 {
                while let Some((i, val)) = changes.pop() {
                    dictionary.constants[i].real_value = Some(val);
                }
            } else {
                // look if every constant has a value
                for constant in &dictionary.constants {
                    if constant.real_value.is_none() {
                        errors.push(ErrType::CannotInitializeConstant(
                            constant.identifier.to_string(),
                        ))
                    }
                }
                break;
            }
        }
        for i in 0..dictionary.constants.len() {
            let constant = &dictionary.constants[i];
            if let Some(val) = analyze_const(&constant.value, dictionary, errors) {
                changes.push((i, val));
            }
        }
        if changes.len() > 0 {
            while let Some((i, val)) = changes.pop() {
                dictionary.constants[i].real_value = Some(val);
            }
        } else {
            // look if every constant has a value
            for constant in &dictionary.constants {
                if constant.real_value.is_none() {
                    errors.push(ErrType::CannotInitializeConstant(
                        constant.identifier.to_string(),
                    ))
                }
            }
        }
    }
    pub fn analyze_const(
        constant: &ValueType,
        dictionary: &Dictionary,
        errors: &mut Vec<ErrType>,
    ) -> Option<ConstValue> {
        match constant {
            ValueType::AnonymousFunction(fun) => Some(ConstValue::Function((*fun).clone())),
            ValueType::Value(val) => {
                let unaries = &val.unary;
                let val = match &val.root.0 {
                    Root::Identifier(val) => match val.as_str() {
                        "true" => Some(ConstValue::Bool(true)),
                        "false" => Some(ConstValue::Bool(false)),
                        _ => {
                            let const_val = dictionary.find_const(&val);
                            if let Some(const_val) = const_val {
                                if let Some(val) = &const_val.real_value {
                                    Some((*val).clone())
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                    },
                    Root::Literal(lit) => ConstValue::from_literal(lit, dictionary, errors),
                    Root::Parenthesis(val) => analyze_const(val, dictionary, errors),
                };
                match val {
                    Some(mut val) => {
                        for op in unaries {
                            val.apply_unary(&op.0)
                        }
                        Some(val)
                    }
                    None => None,
                }
            }
            ValueType::Expression(expression) => {
                let left = if let Some(left) = &expression.left {
                    analyze_const(left, dictionary, errors)
                } else {
                    return None;
                };
                if left.is_none() {
                    return None;
                }
                let left = left.unwrap();
                let right = if let Some(right) = &expression.right {
                    analyze_const(right, dictionary, errors)
                } else {
                    return None;
                };
                if right.is_none() {
                    return None;
                }
                let right = right.unwrap();
                let op = if let Some(op) = &expression.operator {
                    op
                } else {
                    errors.push(ErrType::MissingOperator(expression.line));
                    return None;
                };
                match &op {
                    Operators::Plus => match (&left, &right) {
                        (ConstValue::String(l), ConstValue::String(r)) => {
                            Some(ConstValue::String(l.clone() + r))
                        }
                        _ => {
                            let l = left.into_number();
                            let r = right.into_number();
                            if let (Some(l), Some(r)) = (l, r) {
                                match op {
                                    Operators::Plus => Some(ConstValue::Number(l.0 + r.0)),
                                    Operators::Minus => Some(ConstValue::Number(l.0 - r.0)),
                                    Operators::Star => Some(ConstValue::Number(l.0 * r.0)),
                                    Operators::Slash => Some(ConstValue::Number(l.0 / r.0)),
                                    Operators::Mod => Some(ConstValue::Number(l.0 % r.0)),
                                    _ => None,
                                }
                            } else {
                                None
                            }
                        }
                    },
                    _ => None,
                }
            }
            ValueType::Blank => Some(ConstValue::Undefined),
            _ => None,
        }
    }
    pub fn load_dictionary(
        nodes: &Vec<Node>,
        dictionary: &mut Dictionary,
        errors: &mut Vec<ErrType>,
        file_name: &str,
    ) {
        for node in nodes {
            load_node(node, dictionary, errors, file_name);
        }
    }
    pub fn load_node(
        node: &Node,
        dictionary: &mut Dictionary,
        errors: &mut Vec<ErrType>,
        file_name: &str,
    ) {
        let name = if let Tokens::Text(name) = &node.name {
            name
        } else {
            panic!()
        };
        match name.as_str() {
            "KWEnum" => {
                let mut result = Enum {
                    identifier: get_ident(&node),
                    keys: vec![],
                    methods: vec![],
                    overloads: vec![],
                    line: node.line,
                };
                for enum_value in step_inside_arr(&node, "values") {
                    let n = if let Tokens::Number(n, _) = get_token(&enum_value, "default") {
                        *n as usize
                    } else {
                        let len = result.keys.len() - 1;
                        result.keys[len].1 + 1
                    };
                    let ident = get_ident(&enum_value);
                    for variant in &result.keys {
                        if variant.1 == n {
                            errors.push(ErrType::EnumVariantAssignedNumber(n, enum_value.line))
                        }
                        if variant.0 == ident {
                            errors.push(ErrType::EnumVariantAssignedIdent(
                                ident.to_string(),
                                enum_value.line,
                            ))
                        }
                    }
                    result.keys.push((ident, n, enum_value.line));
                }
                if dictionary.register_id(result.identifier.to_string(), IdentifierKinds::Enum) {
                    dictionary.enums.push(result);
                } else {
                    errors.push(ErrType::ConflictingNames(
                        result.identifier.to_string(),
                        node.line,
                    ))
                }
            }
            "KWType" => {
                let name = get_ident(&node);
                if dictionary.register_id(name.to_string(), IdentifierKinds::Type) {
                    dictionary.types.push(TypeDef {
                        kind: get_type(step_inside_val(&node, "type"), errors, file_name),
                        identifier: name,
                        generics: get_generics_decl(&node, errors),
                        overloads: vec![],
                        methods: vec![],
                        public: public(&node),
                        line: node.line,
                    })
                } else {
                    errors.push(ErrType::ConflictingNames(name.to_string(), node.line))
                }
            }
            "KWStruct" => {
                let mut functions = Vec::new();
                let mut overloads = Vec::new();
                for method in step_inside_arr(&node, "methods") {
                    if let Tokens::Text(txt) = &method.name {
                        match txt.as_str() {
                            "KWOverload" => {
                                overloads.push(get_overload_siginifier(&method, errors, file_name))
                            }
                            "KWFun" => {
                                functions.push(get_fun_siginifier(&method, errors, file_name));
                            }
                            _ => {}
                        }
                    }
                }
                let mut impls = Vec::new();
                for impl_node in step_inside_arr(&node, "impls") {
                    let target =
                        get_nested_ident(&step_inside_val(&impl_node, "identifier"), errors);
                    let mut functions = Vec::new();
                    let mut overloads = Vec::new();
                    for method in step_inside_arr(&impl_node, "methods") {
                        if let Tokens::Text(txt) = &method.name {
                            match txt.as_str() {
                                "KWOverload" => overloads
                                    .push(get_overload_siginifier(&method, errors, file_name)),
                                "KWFun" => {
                                    functions.push(get_fun_siginifier(&method, errors, file_name));
                                }
                                _ => {}
                            }
                        }
                    }
                    impls.push(Implementation {
                        target,
                        functions,
                        overloads,
                        line: impl_node.line,
                    })
                }
                let constructor =
                    if let Some(constructor) = try_step_inside_val(&node, "constructor") {
                        if let Tokens::Text(txt) = &constructor.name {
                            if txt == "KWConstructor" {
                                let mut args = Vec::new();
                                for arg in step_inside_arr(&constructor, "arguments") {
                                    let ident = get_ident(&arg);
                                    let kind =
                                        get_type(step_inside_val(&arg, "type"), errors, file_name);
                                    args.push(Arg {
                                        identifier: ident,
                                        kind,
                                        line: arg.line,
                                    })
                                }
                                let code = if constructor.nodes.contains_key("code") {
                                    codeblock_parser::generate_tree(
                                        step_inside_val(&constructor, "code"),
                                        errors,
                                        file_name,
                                    )
                                } else {
                                    vec![]
                                };
                                let return_type = Some(Kind {
                                    body: TypeBody::Type {
                                        main: vec![get_ident(node)],
                                        generics: Vec::new(),
                                        refs: 0,
                                        nullable: false,
                                        kind: KindType::None,
                                    },
                                    line: node.line,
                                    file: Some(file_name.to_string()),
                                });
                                functions.push(Function {
                                    can_yeet: false,
                                    identifier: Some(get_ident(&node)),
                                    args,
                                    stack_size: None,
                                    location: 0,
                                    instrs_end: 0,
                                    return_type,
                                    generics: Vec::new(),
                                    public: false,
                                    code,
                                    line: constructor.line,
                                    pointers: None,
                                    id: 0,
                                    takes_self: false,
                                });
                                Some(functions.len() - 1)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                let mut result = Struct {
                    identifier: get_ident(node),
                    fields: Vec::new(),
                    generics: get_generics_decl(node, errors),
                    traits: Vec::new(),
                    public: public(&node),
                    memory_layout: Vec::new(),
                    ptrs: 0,
                    line: node.line,
                    functions,
                    overloads,
                    impls,
                    constructor,
                };
                for key in step_inside_arr(node, "keys") {
                    let ident = get_ident(&key);
                    for field in &result.fields {
                        if *field.0 == ident {
                            errors.push(ErrType::StructVariantAssignedIdent(
                                ident.to_string(),
                                field.1.line,
                            ))
                        }
                    }
                    result.fields.push((
                        get_ident(key),
                        get_type(step_inside_val(key, "type"), errors, file_name),
                    ))
                }
                // check for conflicting names
                let mut fnames = Vec::new();
                for field in &result.fields {
                    fnames.push(field.0.clone());
                    if field.0 == result.identifier {
                        errors.push(ErrType::ConflictingNames(field.0.to_string(), field.1.line))
                    }
                }
                let mut mnames = Vec::new();
                for fun in &result.functions {
                    if let Some(ident) = &fun.identifier {
                        mnames.push(ident.clone());
                    }
                }
                for field in &result.fields {
                    for fun in &result.functions {
                        if let Some(ident) = &fun.identifier {
                            if ident == &field.0 {
                                errors.push(ErrType::ConflictingNames(ident.to_string(), fun.line))
                            }
                        }
                    }
                }
                if dictionary.register_id(result.identifier.to_string(), IdentifierKinds::Struct) {
                    dictionary.structs.push(result);
                } else {
                    errors.push(ErrType::ConflictingNames(
                        result.identifier.to_string(),
                        node.line,
                    ))
                }
            }
            "KWImport" => {
                let path = if let Tokens::String(path) = &step_inside_val(&node, "path").name {
                    path.to_string()
                } else {
                    unreachable!("Path not specified");
                };
                let mut kind = ImportKinds::Rd;
                let alias = match try_get_ident(&node) {
                    Some(alias) => alias,
                    None => {
                        let alias = path.split("/").last().unwrap();
                        if alias.starts_with("#") {
                            kind = ImportKinds::Dll;
                            alias[1..].to_string()
                        } else {
                            alias.to_string()
                        }
                    }
                };
                dictionary.imports.push(Import {
                    path,
                    alias: alias.trim_end_matches(".rd").to_string(),
                    line: node.line,
                    kind,
                });
            }
            "KWFun" => {
                let fun = get_fun_siginifier(&node, errors, file_name);
                let name = fun
                    .identifier
                    .clone()
                    .expect("global function cannot be anonymous");
                if dictionary.register_id(String::from(&name), IdentifierKinds::Function) {
                    dictionary.functions.push(fun);
                } else {
                    errors.push(ErrType::ConflictingNames(String::from(&name), node.line))
                }
            }
            "KWLet" => {
                let identifier = get_ident(&node);
                let kind = if let Tokens::Text(txt) = &step_inside_val(node, "type").name {
                    if txt == "type_specifier" {
                        Some(get_type(
                            step_inside_val(step_inside_val(node, "type"), "type"),
                            errors,
                            file_name,
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                };
                if dictionary.register_id(identifier.to_string(), IdentifierKinds::Variable) {
                    dictionary.variables.push(Variable {
                        kind,
                        identifier,
                        location: 0,
                        line: node.line,
                    })
                } else {
                    errors.push(ErrType::ConflictingNames(identifier.to_string(), node.line))
                }
            }
            "KWConst" => {
                let identifier = get_ident(&node);
                if dictionary.register_id(identifier.to_string(), IdentifierKinds::Variable) {
                    dictionary.constants.push(Constant {
                        identifier,
                        location: 0,
                        public: public(&node),
                        value: expression_parser::expr_into_tree(
                            step_inside_val(&node, "expression"),
                            errors,
                            file_name,
                        ),
                        real_value: None,
                        line: node.line,
                    })
                } else {
                    errors.push(ErrType::ConflictingNames(identifier.to_string(), node.line))
                }
                //expression_parser::traverse_da_fokin_value(&expression_parser::expr_into_tree(step_inside_val(&node, "expression"), errors), 0);
                //println!("{:#?}", expression_parser::expr_into_tree(step_inside_val(&node, "expression"), errors));
            }
            "KWImpl" => {
                unreachable!("impl statements moved inside structs");
                /*let ident = get_nested_ident(&step_inside_val(&node, "identifier"), errors);
                let mut functions = Vec::new();
                let mut overloads = Vec::new();
                for method in step_inside_arr(&node, "methods") {
                    if let Tokens::Text(txt) = &method.name {
                        match txt.as_str() {
                            "KWOverload" => {
                                overloads.push(get_overload_siginifier(&method, errors))
                            }
                            "KWFun" => {
                                functions.push(get_fun_siginifier(&method, errors));
                            }
                            _ => {}
                        }
                    }
                }
                dictionary.implementations.push(Implementation {
                    target: ident,
                    functions,
                    overloads,
                    line: node.line,
                })*/
            }
            "KWTrait" => {
                let is_pub = public(&node);
                let identifier = get_ident(&node);
                let mut functions = Vec::new();
                let mut overloads = Vec::new();
                let traits = get_traits(&node, errors);
                for method in step_inside_arr(&node, "methods") {
                    if let Tokens::Text(txt) = &method.name {
                        match txt.as_str() {
                            "KWOverload" => {
                                overloads.push(get_overload_siginifier(&method, errors, file_name))
                            }
                            "KWFun" => {
                                functions.push(get_fun_siginifier(&method, errors, file_name));
                            }
                            _ => {}
                        }
                    }
                }
                if dictionary.register_id(identifier.to_string(), IdentifierKinds::Trait) {
                    dictionary.traits.push(Trait {
                        identifier,
                        methods: functions,
                        overloads,
                        traits,
                        public: is_pub,
                        line: node.line,
                    })
                } else {
                    errors.push(ErrType::ConflictingNames(identifier.to_string(), node.line))
                }
            }
            "expression" => {}
            "KWError" => {
                let ident = get_ident(&node);
                let mut args = Vec::new();
                for arg in step_inside_arr(&node, "args") {
                    let ident = get_ident(&arg);
                    let kind = get_type(&step_inside_val(&arg, "type"), errors, file_name);
                    args.push(Arg {
                        identifier: ident,
                        kind,
                        line: arg.line,
                    })
                }
                let mut fields = Vec::new();
                for field in step_inside_arr(&node, "fields") {
                    let ident = get_ident(&field);
                    let val = step_inside_val(&step_inside_val(&field, "value"), "expression");
                    if let Tokens::Text(txt) = &val.name {
                        match txt.as_str() {
                            "expression" => {
                                let expr =
                                    expression_parser::expr_into_tree(&val, errors, file_name);
                                //expression_parser::traverse_da_fokin_value(&expr, 0);
                                fields.push((ident, ErrorField::Expression(expr)));
                            }
                            "code_block" => {
                                todo!();
                                //fields.push((ident, ErrorField::CodeBlock(code_block_parser::parse_code_block(&val, errors))))
                            }
                            _ => unreachable!("invalid field value"),
                        }
                    }
                }
                if dictionary.register_id(ident.to_string(), IdentifierKinds::Error) {
                    dictionary.errors.push(Error {
                        identifier: ident,
                        args,
                        fields,
                        src_loc: 0,
                        line: node.line,
                    })
                } else {
                    errors.push(ErrType::ConflictingNames(ident.to_string(), node.line))
                }
            }
            "KWUse" => {
                let mut path = Vec::new();
                for part in step_inside_arr(&node, "path") {
                    match &part.name {
                        Tokens::Text(txt) => {
                            path.push(PathPart::Identifier(txt.to_string()));
                        }
                        Tokens::Operator(Operators::Star) => {
                            path.push(PathPart::Asterisk);
                        }
                        _ => {}
                    }
                }
                let alias = try_get_ident(&node);
                dictionary.uses.push(Use {
                    path,
                    alias,
                    line: node.line,
                });
            }
            _ => {}
        }
    }
    pub fn get_traits(node: &Node, errors: &mut Vec<ErrType>) -> Vec<NestedIdent> {
        let mut result = vec![];
        for tr in step_inside_arr(&node, "traits") {
            result.push(get_nested_ident(tr, errors));
        }
        result
    }
    pub fn get_nested_ident(node: &Node, _errors: &mut Vec<ErrType>) -> NestedIdent {
        let mut result = vec![];
        for nd in step_inside_arr(node, "nodes") {
            if let Tokens::Text(txt) = &step_inside_val(nd, "identifier").name {
                result.push(txt.to_string());
            } else {
                panic!()
            }
        }
        result
    }
    pub fn get_overload_siginifier(
        node: &Node,
        errors: &mut Vec<ErrType>,
        file_name: &str,
    ) -> Overload {
        let operator = get_operator(step_inside_val(&node, "op"));
        let generics = get_generics_decl(&node, errors);
        let kind = if let Some(kind) = try_step_inside_val(step_inside_val(&node, "type"), "type") {
            Some(get_type(kind, errors, file_name))
        } else {
            None
        };
        let arg = step_inside_val(&node, "arg");

        // fujj
        let code = if node.nodes.contains_key("code") {
            codeblock_parser::generate_tree(step_inside_val(&node, "code"), errors, file_name)
        } else {
            vec![]
        };

        Overload {
            operator,
            arg: Arg {
                identifier: get_ident(&arg),
                kind: get_type(step_inside_val(&arg, "type"), errors, file_name),
                line: arg.line,
            },
            stack_size: None,
            location: None,
            return_type: kind,
            generics,
            public: public(&node),
            code,
            line: node.line,
        }
    }
    pub fn get_fun_siginifier(node: &Node, errors: &mut Vec<ErrType>, file_name: &str) -> Function {
        let identifier = if node.nodes.contains_key("identifier") {
            Some(get_ident(&node))
        } else {
            None
        };
        let generics = get_generics_decl(&node, errors);
        let kind = if let Some(kind) = try_step_inside_val(step_inside_val(&node, "type"), "type") {
            Some(get_type(kind, errors, file_name))
        } else {
            None
        };
        let mut takes_self = false;
        let mut args: Vec<Arg> = Vec::new();
        for arg in step_inside_arr(node, "arguments") {
            if let Tokens::Text(name) = &arg.name {
                match name.as_str() {
                    "self_arg" => {
                        takes_self = true;
                    }
                    "arg" => {
                        let ident = get_ident(arg);
                        for arg in &args {
                            if arg.identifier == ident {
                                errors.push(ErrType::ConflictingArgsName(
                                    ident.to_string(),
                                    arg.line,
                                ));
                            }
                        }
                        args.push(Arg {
                            identifier: ident,
                            kind: get_type(step_inside_val(&arg, "type"), errors, file_name),
                            line: arg.line,
                        });
                    }
                    _ => {
                        unreachable!()
                    }
                }
            }
        }
        let can_yeet = step_inside_val(&node, "errorable").name
            == Tokens::Operator(crate::lexer::tokenizer::Operators::Not);
        let public = if node.nodes.contains_key("public") {
            public(&node)
        } else {
            true
        };
        /*

        to read the dictionary, you need to do this:

        let mut dict = Dictionary::new();
        load_dictionary(step_inside_arr(step_inside_val(node, "code"), "nodes"), &mut dict, &mut vec![]);
         */

        let code = if node.nodes.contains_key("code") {
            codeblock_parser::generate_tree(step_inside_val(&node, "code"), errors, file_name)
        } else {
            vec![]
        };
        Function {
            can_yeet,
            identifier,
            args,
            stack_size: None,
            location: 0,
            instrs_end: 0,
            return_type: kind,
            generics,
            public: false,
            code,
            line: node.line,
            pointers: None,
            id: 0,
            takes_self,
        }
    }
    pub fn public(node: &Node) -> bool {
        if let Tokens::Text(txt) = &step_inside_val(node, "public").name {
            return txt == "pub";
        }
        false
    }
    pub fn get_operator(node: &Node) -> Tokens {
        step_inside_val(node, "op").name.clone()
    }
    pub fn get_ident(node: &Node) -> String {
        if let Tokens::Text(txt) =
            &step_inside_val(&step_inside_val(&node, "identifier"), "identifier").name
        {
            return txt.to_string();
        }
        panic!();
    }
    pub fn try_get_ident(node: &Node) -> Option<String> {
        if let Some(val) = try_step_inside_val(&step_inside_val(&node, "identifier"), "identifier")
        {
            if let Tokens::Text(txt) = &val.name {
                return Some(txt.to_string());
            }
        }
        None
    }
    pub fn count_refs(node: &Node) -> usize {
        let mut refs = 0;
        if let Some(arr) = try_step_inside_arr(&step_inside_val(&node, "ref"), "refs") {
            for ref_type in arr {
                if let Tokens::Operator(Operators::Ampersant) = ref_type.name {
                    refs += 1;
                }
                if let Tokens::Operator(crate::lexer::tokenizer::Operators::And) = ref_type.name {
                    refs += 2;
                }
            }
        }
        refs
    }
    pub fn get_type(node: &Node, errors: &mut Vec<ErrType>, file_name: &str) -> Kind {
        let nullable = if let Some(val) = try_step_inside_val(node, "optional") {
            val.name == Tokens::Optional
        } else {
            false
        };
        let main = step_inside_val(&node, "main");
        if main.name == Tokens::Text(String::from("function_head")) {
            let fun = get_fun_siginifier(&main, errors, file_name);
            let refs = count_refs(&node);
            return Kind {
                body: TypeBody::Function {
                    args: fun.args,
                    return_type: Box::new(fun.return_type),
                    refs,
                },
                line: node.line,
                file: Some(file_name.to_string()),
            };
        }
        let refs = count_refs(node);
        let main = if let Some(type_ident) =
            try_step_inside_arr(step_inside_val(&node, "main"), "nodes")
        {
            let mut main = Vec::new();
            for path_part in type_ident {
                if let Tokens::Text(txt) = get_token(path_part, "identifier") {
                    main.push(txt.to_string())
                }
            }
            main
        } else {
            let arr_kind = step_inside_val(&step_inside_val(&node, "arr"), "type");
            if let Tokens::Text(txt) = &arr_kind.name {
                if txt == "type" {
                    return Kind {
                        body: TypeBody::Array {
                            type_: Box::new(get_type(
                                step_inside_val(&step_inside_val(&node, "arr"), "type"),
                                errors,
                                file_name,
                            )),
                            size: 0,
                            refs,
                            nullable,
                        },
                        line: node.line,
                        file: Some(file_name.to_string()),
                    };
                } else {
                    unreachable!()
                }
            } else {
                unreachable!("This is a bug in the compiler, please report it to the developers");
            }
        };
        return Kind {
            body: TypeBody::Type {
                main,
                generics: get_generics_expr(node, errors, file_name),
                refs,
                nullable,
                kind: KindType::None,
            },
            line: node.line,
            file: Some(file_name.to_string()),
        };
    }
    pub fn get_generics_expr(
        node: &Node,
        errors: &mut Vec<ErrType>,
        file_name: &str,
    ) -> GenericExpr {
        let mut result = Vec::new();
        if let Some(arr) = try_step_inside_arr(step_inside_val(node, "generic"), "types") {
            for generic_expr in arr {
                result.push(get_type(generic_expr, errors, file_name));
            }
        }
        result
    }
    pub fn get_generics_decl<'a>(node: &'a Node, errors: &mut Vec<ErrType>) -> Vec<GenericDecl> {
        let mut generics = Vec::new();
        if let Some(arr) = try_step_inside_arr(step_inside_val(&node, "generic"), "identifiers") {
            for generic in arr {
                let mut traits = Vec::new();
                for ident in step_inside_arr(generic, "traits") {
                    traits.push(get_nested_ident(&ident, errors));
                }
                generics.push(GenericDecl {
                    identifier: get_ident(generic),
                    traits,
                })
            }
        }
        generics
    }
    pub fn get_token<'a>(node: &'a Node, ident: &'a str) -> &'a Tokens {
        return &step_inside_val(&node, ident).name;
    }
    pub fn step_inside_val<'a>(node: &'a Node, ident: &'a str) -> &'a Node {
        node.nodes.get(ident).unwrap().get_value()
    }
    pub fn try_step_inside_val<'a>(node: &'a Node, ident: &'a str) -> Option<&'a Node> {
        match node.nodes.get(ident) {
            Some(arr) => Some(arr.get_value()),
            None => None,
        }
    }
    pub fn step_inside_arr<'a>(node: &'a Node, ident: &'a str) -> &'a Vec<Node> {
        node.nodes.get(ident).unwrap().get_array()
    }
    pub fn try_step_inside_arr<'a>(node: &'a Node, ident: &'a str) -> Option<&'a Vec<Node>> {
        match node.nodes.get(ident) {
            Some(arr) => Some(arr.get_array()),
            None => None,
        }
    }
    pub fn get_loop_ident(node: &Node) -> Option<String> {
        if let Tokens::Text(txt) = &step_inside_val(&node, "identifier").name {
            if txt == "loop_ident" {
                if let Tokens::String(txt) =
                    &step_inside_val(&step_inside_val(&node, "identifier"), "identifier").name
                {
                    return Some(txt.to_string());
                }
            }
        }
        None
    }
    pub fn get_break_ident(node: &Node) -> Option<String> {
        if let Tokens::String(txt) = &step_inside_val(&node, "identifier").name {
            return Some(txt.to_string());
        }
        None
    }
    #[derive(Debug)]
    pub enum Imports {
        Dll(String, Option<libloader::Dictionary>),
        Rd(String, Option<Dictionary>),
        RSO(String, Option<Dictionary>),
    }
    /// all of the defined types/variables (enum, struct, function) in the current scope will be registered here
    #[derive(Debug)]
    pub struct Dictionary {
        pub functions: Vec<Function>,
        pub types: Vec<TypeDef>,
        pub enums: Vec<Enum>,
        pub structs: Vec<Struct>,
        pub variables: Vec<Variable>,
        pub constants: Vec<Constant>,
        pub identifiers: Vec<(String, IdentifierKinds)>,
        pub imports: Vec<Import>,
        pub traits: Vec<Trait>,
        pub errors: Vec<Error>,
        pub uses: Vec<Use>,
    }
    impl Dictionary {
        pub fn find_const(&self, name: &str) -> Option<&Constant> {
            for constant in &self.constants {
                if constant.identifier == name {
                    return Some(constant);
                }
            }
            None
        }
    }
    #[derive(Debug)]
    pub struct Import {
        pub path: String,
        pub alias: String,
        pub line: Line,
        pub kind: ImportKinds,
    }
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub enum ImportKinds {
        Dll,
        Rd,
    }
    #[derive(Debug)]
    pub struct Trait {
        pub identifier: String,
        pub methods: Vec<Function>,
        pub overloads: Vec<Overload>,
        // dependences
        pub traits: Vec<NestedIdent>,
        pub public: bool,
        pub line: Line,
    }
    #[derive(Debug, Clone)]
    pub enum IdentifierKinds {
        Function,
        Type,
        Enum,
        Struct,
        Variable,
        Namespace,
        Trait,
        Error,
    }
    impl std::fmt::Display for IdentifierKinds {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                IdentifierKinds::Function => write!(f, "function"),
                IdentifierKinds::Type => write!(f, "type"),
                IdentifierKinds::Enum => write!(f, "enum"),
                IdentifierKinds::Struct => write!(f, "struct"),
                IdentifierKinds::Variable => write!(f, "variable"),
                IdentifierKinds::Namespace => write!(f, "namespace"),
                IdentifierKinds::Trait => write!(f, "trait"),
                IdentifierKinds::Error => write!(f, "error"),
            }
        }
    }
    #[derive(Debug)]
    pub struct TypeDef {
        pub kind: Kind,
        pub identifier: String,
        pub generics: Vec<GenericDecl>,
        pub public: bool,
        pub overloads: Vec<Overload>,
        pub methods: Vec<Function>,
        pub line: Line,
    }
    #[derive(Debug, Clone)]
    pub struct GenericDecl {
        pub identifier: String,
        pub traits: Vec<NestedIdent>,
    }
    #[derive(Debug)]
    pub struct Error {
        pub identifier: String,
        pub src_loc: usize,
        pub fields: Vec<(String, ErrorField)>,
        pub args: Vec<Arg>,
        pub line: Line,
    }
    #[derive(Debug)]
    pub enum ErrorField {
        Expression(expression_parser::ValueType),
        CodeBlock(Vec<codeblock_parser::Nodes>),
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct Arg {
        pub identifier: String,
        pub kind: Kind,
        pub line: Line,
    }
    impl Arg {
        pub fn from_barg(barg: &(String, Kind, libloader::MemoryTypes, Line)) -> Arg {
            Arg {
                identifier: barg.0.clone(),
                kind: barg.1.clone(),
                line: barg.3,
            }
        }
    }
    #[derive(Debug, Clone)]
    pub struct Use {
        path: Vec<PathPart>,
        alias: Option<String>,
        line: Line,
    }
    #[derive(Debug, Clone)]
    pub enum PathPart {
        Identifier(String),
        Asterisk,
    }
    #[derive(Debug, Clone)]
    pub struct Function {
        /// function identifiers will be changed to allow for function overload
        /// name mangler rules: "{identifier}:{args.foreach("{typeof}:")}"
        /// example:
        /// fun myFun(n: int, type: char): int
        /// fun nothing()
        /// translates to:
        /// "myFun:int:char"
        /// "nothing:"
        pub identifier: Option<String>,
        /// type of args in order
        pub args: Vec<Arg>,
        /// size needed to allocate on stack while function call (args.len() included)
        pub stack_size: Option<usize>,
        /// location in bytecode, so runtime knows where to jump
        pub location: usize,
        pub instrs_end: usize,
        pub return_type: Option<Kind>,
        pub can_yeet: bool,
        pub generics: Vec<GenericDecl>,
        pub public: bool,
        pub code: Vec<codeblock_parser::Nodes>,
        pub line: Line,
        pub pointers: Option<usize>,
        pub id: usize,
        pub takes_self: bool,
    }
    /// used to correct function calls
    #[derive(Debug, Clone)]
    pub struct Correction {
        pub location: usize,
        pub function: InnerPath,
    }
    #[derive(Debug)]
    pub struct Overload {
        pub operator: Tokens,
        /// type of args in order
        pub arg: Arg,
        /// size needed to allocate on stack while function call (args.len() included)
        pub stack_size: Option<usize>,
        /// location in bytecode, so runtime knows where to jump
        pub location: Option<usize>,
        pub return_type: Option<Kind>,
        pub generics: Vec<GenericDecl>,
        pub public: bool,
        pub code: Vec<codeblock_parser::Nodes>,
        pub line: Line,
    }
    #[derive(Debug)]
    pub struct Enum {
        pub identifier: String,
        /// enum values and their offset
        /// enum ErrCode { Continue = 100, SwitchingProtocols, ..., Ok = 200, ... }
        pub keys: Vec<(String, usize, Line)>,
        pub methods: Vec<Function>,
        pub overloads: Vec<Overload>,
        pub line: Line,
    }
    pub type NestedIdent = Vec<String>;
    #[derive(Debug)]
    pub struct Struct {
        pub generics: Vec<GenericDecl>,
        pub identifier: String,
        pub fields: Vec<(String, Kind)>,
        pub traits: Vec<NestedIdent>,
        pub public: bool,
        pub memory_layout: Vec<(String, usize)>,
        /// number of pointers in struct
        pub ptrs: usize,
        pub line: Line,
        pub impls: Vec<Implementation>,
        pub functions: Vec<Function>,
        pub overloads: Vec<Overload>,
        /// index of function that is a constructor
        pub constructor: Option<usize>,
    }
    impl Struct {
        pub fn get_field(&self, name: &str) -> Option<(crate::codegen::CompoundField, usize)> {
            for field in self.fields.iter().enumerate() {
                if field.1 .0 == name {
                    return Some((
                        crate::codegen::CompoundField::Field(field.1 .0.clone()),
                        field.0,
                    ));
                }
            }
            for method in &self.functions {
                if let Some(ident) = &method.identifier {
                    if ident == name {
                        return Some((
                            crate::codegen::CompoundField::Method(
                                method.identifier.clone().unwrap(),
                            ),
                            0,
                        ));
                    }
                }
            }
            for overload in &self.overloads {
                if &overload.arg.identifier == name {
                    return Some((
                        crate::codegen::CompoundField::OverloadedOperator(todo!()),
                        0,
                    ));
                }
            }

            None
        }
    }
    #[derive(Debug)]
    pub struct Implementation {
        pub target: NestedIdent,
        pub functions: Vec<Function>,
        pub overloads: Vec<Overload>,
        pub line: Line,
    }
    #[derive(Debug)]
    pub struct Variable {
        pub kind: Option<Kind>,
        pub identifier: String,
        /// location on stack
        pub location: usize,
        pub line: Line,
    }
    #[derive(Debug)]
    pub struct Constant {
        pub identifier: String,
        /// location on stack
        pub location: usize,
        pub public: bool,
        pub value: expression_parser::ValueType,
        pub real_value: Option<ConstValue>,
        pub line: Line,
    }
    #[derive(Debug, Clone)]
    pub enum ConstValue {
        Number(f64),
        Int(i64),
        Float(f64),
        Char(char),
        Bool(bool),
        Usize(usize),
        Function(Function),
        String(String),
        Null,
        Array(Vec<ConstValue>),
        Undefined,
    }
    impl ConstValue {
        pub fn from_literal(
            literal: &expression_parser::Literals,
            dictionary: &Dictionary,
            errors: &mut Vec<ErrType>,
        ) -> Option<ConstValue> {
            match &literal {
                expression_parser::Literals::Number(num) => {
                    if let Tokens::Number(num, kind) = *num {
                        match kind {
                            'f' => Some(ConstValue::Float(num as f64)),
                            'u' => Some(ConstValue::Int(num as i64 as i64)),
                            'n' => Some(ConstValue::Number(num)),
                            'i' => Some(ConstValue::Int(num as i64 as i64)),
                            'c' => Some(ConstValue::Char(num as u8 as char)),
                            _ => None,
                        }
                    } else {
                        None
                    }
                }
                expression_parser::Literals::Char(c) => Some(ConstValue::Char(*c)),
                expression_parser::Literals::Array(arr_rule) => match arr_rule {
                    expression_parser::ArrayRule::Fill { value, size } => {
                        let mut arr = Vec::new();
                        let s = analyze_const(&size, dictionary, errors);
                        let v = analyze_const(&value, dictionary, errors)
                            .unwrap_or(ConstValue::Undefined);
                        if let Some(n) = s {
                            if let Some(n_data) = n.into_number() {
                                let (i, _) = n_data;
                                for _ in 0..i as usize {
                                    arr.push(v.clone());
                                }
                            }
                            Some(ConstValue::Array(arr))
                        } else {
                            None
                        }
                    }
                    expression_parser::ArrayRule::Explicit(values) => {
                        let mut arr = Vec::new();
                        for v in values {
                            let c = analyze_const(&v, dictionary, errors);
                            arr.push(c.unwrap_or(ConstValue::Undefined));
                        }
                        Some(ConstValue::Array(arr))
                    }
                },
                expression_parser::Literals::String(str) => Some(ConstValue::String(str.clone())),
            }
        }
        pub fn into_number(&self) -> Option<(f64, char)> {
            match self {
                ConstValue::Number(f) => Some((*f, 'n')),
                ConstValue::Char(c) => Some((*c as usize as f64, 'c')),
                ConstValue::Int(i) => Some((*i as f64, 'i')),
                ConstValue::Float(f) => Some((*f, 'f')),
                ConstValue::Usize(i) => Some((*i as f64, 'u')),
                _ => None,
            }
        }
        pub fn is_array(&self) -> bool {
            match self {
                ConstValue::Array(_) => true,
                _ => false,
            }
        }
        pub fn apply_unary(&mut self, op: &Operators) {
            match self {
                ConstValue::Number(f) => match op {
                    Operators::Minus => *f *= -1.0,
                    _ => (),
                },
                ConstValue::Int(i) => match op {
                    Operators::Minus => *i *= -1,
                    _ => (),
                },
                ConstValue::Float(f) => match op {
                    Operators::Minus => *f *= -1.0,
                    _ => (),
                },
                ConstValue::Bool(b) => match op {
                    Operators::Not => *b = !*b,
                    _ => (),
                },
                _ => {}
            }
        }
        pub fn vm_partial_eq(&self, other: &runtime::runtime_types::Types) -> bool {
            match self {
                ConstValue::Number(n) => match other {
                    runtime::runtime_types::Types::Float(f) => n == f,
                    _ => false,
                },
                ConstValue::Int(n) => match other {
                    runtime::runtime_types::Types::Int(i) => n == i,
                    _ => false,
                },
                ConstValue::Float(n) => match other {
                    runtime::runtime_types::Types::Float(f) => n == f,
                    _ => false,
                },
                ConstValue::Char(n) => match other {
                    runtime::runtime_types::Types::Char(c) => n == c,
                    _ => false,
                },
                ConstValue::Bool(n) => match other {
                    runtime::runtime_types::Types::Bool(b) => n == b,
                    _ => false,
                },
                ConstValue::Usize(n) => match other {
                    runtime::runtime_types::Types::Usize(u) => n == u,
                    _ => false,
                },
                ConstValue::Null => match other {
                    runtime::runtime_types::Types::Null => true,
                    _ => false,
                },
                _ => false,
            }
        }
        pub fn to_runtime(&self) -> runtime_types::Types {
            match self {
                ConstValue::Number(n) => runtime_types::Types::Float(*n),
                ConstValue::Int(n) => runtime_types::Types::Int(*n),
                ConstValue::Float(n) => runtime_types::Types::Float(*n),
                ConstValue::Char(n) => runtime_types::Types::Char(*n),
                ConstValue::Bool(n) => runtime_types::Types::Bool(*n),
                ConstValue::Usize(n) => runtime_types::Types::Usize(*n),
                ConstValue::Null => runtime_types::Types::Null,
                _ => runtime_types::Types::Null,
            }
        }
        pub fn gen_type(&self) -> Option<ShallowType> {
            let res = match self {
                ConstValue::Number(_) => ShallowType {
                    is_fun: None,
                    array_depth: 0,
                    refs: 0,
                    main: vec![String::from("int")],
                    generics: Vec::new(),
                    line: Line { line: 0, column: 0 },
                    nullable: false,
                    file: None,
                    kind: KindType::Primitive,
                },
                ConstValue::Int(_) => ShallowType {
                    is_fun: None,
                    array_depth: 0,
                    refs: 0,
                    main: vec![String::from("int")],
                    generics: Vec::new(),
                    line: Line { line: 0, column: 0 },
                    nullable: false,
                    file: None,
                    kind: KindType::Primitive,
                },
                ConstValue::Float(_) => ShallowType {
                    is_fun: None,
                    array_depth: 0,
                    refs: 0,
                    main: vec![String::from("float")],
                    generics: Vec::new(),
                    line: Line { line: 0, column: 0 },
                    nullable: false,
                    file: None,
                    kind: KindType::Primitive,
                },
                ConstValue::Char(_) => ShallowType {
                    is_fun: None,
                    array_depth: 0,
                    refs: 0,
                    main: vec![String::from("char")],
                    generics: Vec::new(),
                    line: Line { line: 0, column: 0 },
                    nullable: false,
                    file: None,
                    kind: KindType::Primitive,
                },
                ConstValue::Bool(_) => ShallowType {
                    is_fun: None,
                    array_depth: 0,
                    refs: 0,
                    main: vec![String::from("bool")],
                    generics: Vec::new(),
                    line: Line { line: 0, column: 0 },
                    nullable: false,
                    file: None,
                    kind: KindType::Primitive,
                },
                ConstValue::Usize(_) => ShallowType {
                    is_fun: None,
                    array_depth: 0,
                    refs: 0,
                    main: vec![String::from("uint")],
                    generics: Vec::new(),
                    line: Line { line: 0, column: 0 },
                    nullable: false,
                    file: None,
                    kind: KindType::Primitive,
                },
                ConstValue::String(_) => ShallowType {
                    is_fun: None,
                    array_depth: 0,
                    refs: 0,
                    main: vec![String::from("string")],
                    generics: Vec::new(),
                    line: Line { line: 0, column: 0 },
                    nullable: false,
                    file: None,
                    kind: KindType::Primitive,
                },
                ConstValue::Null => ShallowType {
                    is_fun: None,
                    array_depth: 0,
                    refs: 0,
                    main: vec![String::from("null")],
                    generics: Vec::new(),
                    line: Line { line: 0, column: 0 },
                    nullable: false,
                    file: None,
                    kind: KindType::Primitive,
                },
                ConstValue::Function(_) => todo!(),
                ConstValue::Array(arr) => {
                    let mut res = ShallowType {
                        is_fun: None,
                        array_depth: 0,
                        refs: 0,
                        main: vec![],
                        generics: Vec::new(),
                        line: Line { line: 0, column: 0 },
                        nullable: false,
                        file: None,
                        kind: KindType::Primitive,
                    };
                    res.array_depth = 1;
                    res
                }
                ConstValue::Undefined => ShallowType::empty(),
            };
            Some(res)
        }
    }
    pub type GenericExpr = Vec<Kind>;

    #[derive(Clone)]
    pub struct ShallowType {
        pub is_fun: Option<Box<Function>>,
        // if 0 then not array, if 1 then array, if 2 then array of arrays, etc...
        pub array_depth: usize,
        pub refs: usize,
        pub main: NestedIdent,
        pub generics: GenericExpr,
        pub line: Line,
        pub nullable: bool,
        pub file: Option<String>,
        pub kind: KindType,
    }
    // print formating
    impl std::fmt::Debug for ShallowType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            for _ in 0..self.refs {
                write!(f, "&")?;
            }
            if let Some(fun) = &self.is_fun {
                write!(f, "{}", fun.identifier.as_ref().unwrap())?;
                write!(f, "(")?;
                for (i, arg) in fun.args.iter().enumerate() {
                    write!(f, "{:?}", arg.kind)?;
                    if i != fun.args.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ")")?;
                return Ok(());
            }
            for _ in 0..self.array_depth {
                write!(f, "[")?;
            }
            for (i, part) in self.main.iter().enumerate() {
                write!(f, "{}", part)?;
                if i != self.main.len() - 1 {
                    write!(f, ".")?;
                }
            }
            for _ in 0..self.array_depth {
                write!(f, "]")?;
            }
            if !self.generics.is_empty() {
                write!(f, "<")?;
                for (i, gen) in self.generics.iter().enumerate() {
                    write!(f, "{:?}", gen)?;
                    if i != self.generics.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ">")?;
            }
            if self.nullable {
                write!(f, "?")?;
            }
            Ok(())
        }
    }
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum KindType {
        Struct,
        Enum,
        Trait,
        Fun,
        Primitive,
        UserData,
        Error,
        BinFun,
        SelfRef,
        None,
    }
    impl KindType {
        pub fn to_import_kind(&self) -> Option<ImportKinds> {
            match self {
                KindType::Struct => Some(ImportKinds::Rd),
                KindType::Enum => Some(ImportKinds::Rd),
                KindType::Trait => Some(ImportKinds::Rd),
                KindType::Fun => Some(ImportKinds::Rd),
                KindType::Primitive => None,
                KindType::UserData => Some(ImportKinds::Dll),
                KindType::Error => None,
                KindType::BinFun => Some(ImportKinds::Dll),
                KindType::SelfRef => None,
                KindType::None => None,
            }
        }
    }
    macro_rules! display_simple {
        ($this: ident) => {
            format!("{:?}", $this)
                .trim_start_matches("&")
                .trim_end_matches("?")
                .to_string()
        };
    }

    impl ShallowType {
        pub fn empty() -> Self {
            ShallowType {
                is_fun: None,
                array_depth: 0,
                refs: 0,
                main: vec![],
                generics: vec![],
                line: Line { line: 0, column: 0 },
                nullable: false,
                file: None,
                kind: KindType::None,
            }
        }
        pub fn null() -> Self {
            ShallowType {
                is_fun: None,
                array_depth: 0,
                refs: 0,
                main: vec!["null".to_string()],
                generics: vec![],
                line: Line { line: 0, column: 0 },
                nullable: false,
                file: None,
                kind: KindType::Primitive,
            }
        }
        pub fn get_ident(&self) -> &str {
            &self.main[self.main.len() - 1]
        }
        pub fn cmp(&self, other: &Self) -> TypeComparison {
            if self.nullable && other.is_null() {
                return TypeComparison::Equal;
            }
            // check if both have the same refs and if not return difference in refs
            if self.refs != other.refs {
                return TypeComparison::ReferenceDiff(self.refs as i32 - other.refs as i32);
            }
            // check if both are the same array depth
            if self.array_depth != other.array_depth {
                return TypeComparison::NotEqual;
            }
            if self.main.last() != other.main.last() {
                return TypeComparison::NotEqual;
            }
            /*if self.generics.len() != other.generics.len() {
                return TypeComparison::NotEqual;
            }
            for (i, gen) in self.generics.iter().enumerate() {
                if gen != &other.generics[i] {
                    return TypeComparison::NotEqual;
                }
            }*/
            if self.file != other.file && !self.is_primitive() && !other.is_primitive() {
                println!("{} != {}", display_simple!(self), display_simple!(other));
                println!(
                    "{} != {}",
                    self.file.as_ref().unwrap(),
                    other.file.as_ref().unwrap()
                );
                return TypeComparison::NotEqual;
            }
            TypeComparison::Equal
        }
        pub fn is_null(&self) -> bool {
            self.is_primitive() && format!("{:?}", self) == "null"
        }
        pub fn is_number(&self) -> bool {
            let temp = format!("{:?}", self);
            self.is_primitive()
                && (temp == "int" || temp == "float" || temp == "char" || temp == "uint")
        }
        pub fn is_string(&self) -> bool {
            self.is_primitive() && format!("{:?}", self) == "string"
        }
        pub fn is_primitive(&self) -> bool {
            let name = &format!("{:?}", self)
                .trim_start_matches("&")
                .trim_end_matches("?")
                .to_string();
            self.kind == KindType::Primitive
                || (name == "int"
                    || name == "float"
                    || name == "char"
                    || name == "uint"
                    || name == "bool"
                    || name == "null"
                    || name == "string")
        }
        pub fn is_primitive_simple(&self) -> bool {
            let name = &format!("{:?}", self);
            self.kind == KindType::Primitive
                || (name == "int"
                    || name == "float"
                    || name == "char"
                    || name == "uint"
                    || name == "bool"
                    || name == "null"
                    || name == "string")
        }
        pub fn core_is_primitive(&self) -> bool {
            let mut kind = self.clone();
            kind.array_depth = 0;
            kind.refs = 0;
            kind.nullable = false;
            kind.generics = Vec::new();
            let name = &format!("{:?}", self);
            name == "int"
                || name == "float"
                || name == "char"
                || name == "uint"
                || name == "bool"
                || name == "null"
                || name == "string"
                || name == "null"
                || name == "string"
        }
        pub fn is_bool(&self) -> bool {
            self.is_primitive() && format!("{:?}", self) == "bool"
        }
        pub fn into_runtime(&self) -> Option<runtime_types::Types> {
            if self.kind != KindType::Primitive {
                return None;
            }
            if self.array_depth != 0 {
                return None;
            }
            let res = match format!("{:?}", self).as_str() {
                "int" => runtime_types::Types::Int(0),
                "float" => runtime_types::Types::Float(0.0),
                "char" => runtime_types::Types::Char(' '),
                "uint" => runtime_types::Types::Usize(0),
                "bool" => runtime_types::Types::Bool(false),
                "null" => runtime_types::Types::Null,
                _ => runtime_types::Types::Null,
            };
            Some(res)
        }
        pub fn into_const(&self) -> Option<ConstValue> {
            if !self.is_primitive() {
                return None;
            }
            if self.array_depth != 0 {
                return None;
            }
            let res = match format!("{:?}", self).as_str() {
                "int" => ConstValue::Int(0),
                "float" => ConstValue::Float(0.0),
                "char" => ConstValue::Char(' '),
                "uint" => ConstValue::Usize(0),
                "bool" => ConstValue::Bool(false),
                "null" => ConstValue::Null,
                _ => ConstValue::Undefined,
            };
            Some(res)
        }
        pub fn from_struct(ident: String, file: String, line: Line) -> Self {
            ShallowType {
                is_fun: None,
                array_depth: 0,
                refs: 0,
                main: vec![ident],
                generics: Vec::new(),
                line,
                nullable: false,
                file: Some(file),
                kind: KindType::Struct,
            }
        }
        pub fn from_userdata(ident: String, file: String, line: Line) -> Self {
            ShallowType {
                is_fun: None,
                array_depth: 0,
                refs: 0,
                main: vec![ident],
                generics: Vec::new(),
                line,
                nullable: false,
                file: Some(file),
                kind: KindType::UserData,
            }
        }
        pub fn from_enum(enum_: &Enum, file: String) -> Self {
            ShallowType {
                is_fun: None,
                array_depth: 0,
                refs: 0,
                main: vec![enum_.identifier.clone()],
                generics: Vec::new(),
                line: enum_.line,
                nullable: false,
                file: Some(file),
                kind: KindType::Enum,
            }
        }
        pub fn from_trait(trait_: &Trait, file: String) -> Self {
            ShallowType {
                is_fun: None,
                array_depth: 0,
                refs: 0,
                main: vec![trait_.identifier.clone()],
                generics: Vec::new(),
                line: trait_.line,
                nullable: false,
                file: Some(file),
                kind: KindType::Trait,
            }
        }
        pub fn from_fun(fun: &Function, file: String) -> Self {
            let main = vec![fun.identifier.as_ref().unwrap().clone()];
            ShallowType {
                is_fun: None,
                array_depth: 0,
                refs: 0,
                main,
                generics: Vec::new(),
                line: fun.line,
                nullable: false,
                file: Some(file),
                kind: KindType::Fun,
            }
        }
        pub fn bfrom_fun(fun: &libloader::Function, file: String) -> Self {
            let main = vec![fun.name.clone()];
            ShallowType {
                is_fun: None,
                array_depth: 0,
                refs: 0,
                main,
                generics: Vec::new(),
                line: Line { line: 0, column: 0 },
                nullable: false,
                file: Some(file),
                kind: KindType::BinFun,
            }
        }
    }

    pub struct ShTypeBuilder {
        pub is_fun: Option<Box<Function>>,
        pub array_depth: usize,
        pub refs: usize,
        pub path: NestedIdent,
        pub generics: GenericExpr,
        pub line: Line,
        pub name: String,
        pub nullable: bool,
        pub file: Option<String>,
        pub kind: KindType,
    }

    impl ShTypeBuilder {
        pub fn new() -> Self {
            ShTypeBuilder {
                is_fun: None,
                array_depth: 0,
                refs: 0,
                path: vec![],
                generics: vec![],
                line: Line { line: 0, column: 0 },
                name: String::new(),
                nullable: false,
                file: None,
                kind: KindType::None,
            }
        }
        pub fn build(mut self) -> ShallowType {
            let main = {
                let mut main = self.path;
                main.push(self.name);
                main
            };
            let len = main.len();
            let first = &main[0];
            if len == 1
                && first == "int"
                && first == "uint"
                && first == "float"
                && first == "char"
                && first == "bool"
                && first == "string"
                && first == "null"
            {
                self.kind = KindType::Primitive;
            }
            ShallowType {
                is_fun: self.is_fun,
                array_depth: self.array_depth,
                refs: self.refs,
                main,
                generics: self.generics,
                line: self.line,
                nullable: self.nullable,
                file: self.file,
                kind: self.kind,
            }
        }
        pub fn set_kind(mut self, kind: KindType) -> Self {
            self.kind = kind;
            self
        }
        pub fn set_file(mut self, file: String) -> Self {
            self.file = Some(file);
            self
        }
        pub fn set_nullable(mut self, nullable: bool) -> Self {
            self.nullable = nullable;
            self
        }
        pub fn set_fun(mut self, fun: Function) -> Self {
            self.is_fun = Some(Box::new(fun));
            self
        }
        pub fn set_array(mut self, array: usize) -> Self {
            self.array_depth = array;
            self
        }
        pub fn set_refs(mut self, refs: usize) -> Self {
            self.refs = refs;
            self
        }
        pub fn set_path(mut self, path: NestedIdent) -> Self {
            self.path = path;
            self
        }
        pub fn set_name(mut self, name: &str) -> Self {
            self.name = name.to_string();
            self
        }
        pub fn set_generics(mut self, generics: GenericExpr) -> Self {
            self.generics = generics;
            self
        }
        pub fn set_line(mut self, line: Line) -> Self {
            self.line = line;
            self
        }
    }

    impl Dictionary {
        pub fn new() -> Self {
            Dictionary {
                functions: vec![],
                types: vec![],
                enums: vec![],
                structs: vec![],
                variables: vec![],
                constants: vec![],
                identifiers: vec![],
                imports: vec![],
                traits: vec![],
                errors: vec![],
                uses: vec![],
            }
        }
        pub fn index_of(&self, identifier: String) -> Option<usize> {
            let mut i = 0;
            loop {
                if i >= self.identifiers.len() {
                    return None;
                }
                if self.identifiers[i].0 == identifier {
                    return Some(i);
                }
                i += 1;
            }
        }
        pub fn type_of(&self, identifier: &str) -> Option<&IdentifierKinds> {
            for (ident, kind) in &self.identifiers {
                if ident == identifier {
                    return Some(kind);
                }
            }
            None
        }
        pub fn register_id(&mut self, name: String, kind: IdentifierKinds) -> bool {
            if self.contains(&name) {
                return false;
            }
            self.identifiers.push((name, kind));
            true
        }
        pub fn force_id(&mut self, name: String, kind: IdentifierKinds) {
            self.identifiers.push((name, kind));
        }
        pub fn contains(&self, name: &String) -> bool {
            for id in &self.identifiers {
                if id.0 == *name {
                    return true;
                }
            }
            false
        }
    }
    #[derive(Debug)]
    pub enum TypeComparison {
        /// types are equal
        Equal,
        /// types are not equal
        NotEqual,
        /// types are not equal, but they are compatible
        Compatible,
        /// types will be equal after referencing or dereferencing
        ReferenceDiff(i32),
        /// both are arrays, but they have different lengths
        /// len1 len2
        ArrayDiff(usize, usize),
        /// The other type is null while this doesnt allow null
        NotNullable,
    }
    impl TypeComparison {
        pub fn is_equal(&self) -> bool {
            match self {
                TypeComparison::Equal => true,
                _ => false,
            }
        }
        pub fn is_not_equal(&self) -> bool {
            match self {
                TypeComparison::NotEqual => true,
                TypeComparison::ArrayDiff(_, _) => true,
                TypeComparison::ReferenceDiff(_) => true,
                TypeComparison::NotNullable => true,
                _ => false,
            }
        }
        pub fn is_compatible(&self) -> bool {
            match self {
                TypeComparison::Compatible => true,
                TypeComparison::Equal => true,
                _ => false,
            }
        }
        pub fn is_reference_diff(&self) -> bool {
            match self {
                TypeComparison::ReferenceDiff(_) => true,
                _ => false,
            }
        }
        pub fn is_array_diff(&self) -> bool {
            match self {
                TypeComparison::ArrayDiff(_, _) => true,
                _ => false,
            }
        }
    }
}
pub mod AnalyzationError {
    use crate::{expression_parser, tree_walker::tree_walker::Line};

    use super::dictionary::IdentifierKinds;

    #[derive(Debug)]
    pub enum ErrType {
        /// assigned_number line col | occurs when you try to assign same number to two or more enum variants
        EnumVariantAssignedNumber(usize, Line),
        /// variant_ident line col | occurs when you try to assign same identifier to two or more enum variants
        EnumVariantAssignedIdent(String, Line),
        /// name | occurs when you try to assign same identifier twice
        ConflictingNames(String, Line),
        /// name | occurs when you try to assign same identifier for two or more arguments
        ConflictingArgsName(String, Line),
        /// name kind | occurs when you try to implement on non implementable identifier (implementable: enum, struct, type)
        BadImpl(String, IdentifierKinds, Line),
        /// name kind | occurs when you try to use identifier that has not been declared
        NonExistentIdentifier(String, Line),
        /// field line col | occurs when you try to assign same identifier to two or more struct fields
        StructVariantAssignedIdent(String, Line),
        /// transform_error | occurs when there is an error in expression
        TreeTransformError(expression_parser::TreeTransformError, Line),
        /// invalid_register | occurs when you try to use register that does not exist
        InvalidRegister(String, Line),
        /// invalid_constant | occurs when you try to use constant that is not supported in rust libraries
        InvalidConstant(crate::lexer::tokenizer::Tokens, Line),
        /// import_path | occurs when you try to import file that does not exist
        ImportPathDoesNotExist(String, Line),
        /// not_code_block | occurs when you try to use code block that is not code block (probably wont happen tho)
        NotCodeBlock(Line),
        /// not_operator | occurs when you try to use operator that is not operator (probably wont happen tho)
        NotOperator(Line),
        /// cannot_initialize_constant | occurs when you try to initialize constant with something that is not constant
        CannotInitializeConstant(String),
        /// missong_operator | occurs when expression expects operator but there is none
        MissingOperator(Line),
    }

    impl std::fmt::Display for ErrType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                ErrType::EnumVariantAssignedNumber(num, line) => {
                    write!(f, "enum variant assigned number {num} at {line}")
                }
                ErrType::EnumVariantAssignedIdent(ident, line) => {
                    write!(f, "enum variant assigned identifier {ident} at {line}")
                }
                ErrType::ConflictingNames(ident, line) => {
                    write!(f, "conflicting names {ident} at {line}")
                }
                ErrType::ConflictingArgsName(ident, line) => {
                    write!(f, "conflicting args name {ident} at {line}")
                }
                ErrType::BadImpl(ident, kind, line) => {
                    write!(f, "bad impl {ident} {kind} at {line}")
                }
                ErrType::NonExistentIdentifier(ident, line) => {
                    write!(f, "non existent identifier {ident} at {line}")
                }
                ErrType::StructVariantAssignedIdent(ident, line) => {
                    write!(f, "struct variant assigned identifier {ident} at {line}")
                }
                ErrType::TreeTransformError(err, line) => {
                    write!(f, "tree transform error {err} at {line}")
                }
                ErrType::InvalidRegister(ident, line) => {
                    write!(f, "invalid register {ident} at {line}")
                }
                ErrType::InvalidConstant(token, line) => {
                    write!(f, "invalid constant {token} at {line}")
                }
                ErrType::ImportPathDoesNotExist(path, line) => {
                    write!(f, "import path does not exist {path} at {line}")
                }
                ErrType::NotCodeBlock(line) => {
                    write!(f, "not code block at {line}")
                }
                ErrType::NotOperator(line) => {
                    write!(f, "not operator at {line}")
                }
                ErrType::CannotInitializeConstant(ident) => {
                    write!(f, "cannot initialize constant {ident}")
                }
                ErrType::MissingOperator(line) => {
                    write!(f, "missing operator at {line}")
                }
            }
        }
    }
}

use std::borrow::BorrowMut;

use super::dictionary::*;
use crate::{codegen::InnerPath, libloader, tree_walker::tree_walker::Line};
#[derive(Clone, PartialEq)]
pub struct Kind {
    pub body: TypeBody,
    pub line: Line,
    pub file: Option<String>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum TypeBody {
    Function {
        args: Vec<Arg>,
        return_type: Box<Option<Kind>>,
        refs: usize,
    },
    Type {
        refs: usize,
        main: NestedIdent,
        generics: GenericExpr,
        nullable: bool,
        kind: KindType,
    },
    Generic {
        identifier: String,
        constraints: Vec<InnerPath>,
    },
    Array {
        type_: Box<Kind>,
        size: usize,
        refs: usize,
        nullable: bool,
    },
    Void,
}

impl std::fmt::Debug for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.body {
            TypeBody::Function {
                args,
                return_type,
                refs,
            } => {
                write!(f, "fun(")?;
                for (i, arg) in args.iter().enumerate() {
                    write!(f, "{:?}", arg)?;
                    if i != args.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ") -> ")?;
                if let Some(kind) = return_type.as_ref() {
                    write!(f, "{:?}", kind)?;
                } else {
                    write!(f, "void")?;
                }
                write!(f, " refs: {}", refs)?;
            }
            TypeBody::Type {
                refs,
                main,
                generics,
                nullable,
                kind,
            } => {
                for _ in 0..*refs {
                    write!(f, "&")?;
                }
                write!(f, "{:?}", main)?;
                if !generics.is_empty() {
                    write!(f, "<")?;
                    for (i, gen) in generics.iter().enumerate() {
                        write!(f, "{:?}", gen)?;
                        if i != generics.len() - 1 {
                            write!(f, ", ")?;
                        }
                    }
                    write!(f, ">")?;
                }
                if *nullable {
                    write!(f, "?")?;
                }
            }
            TypeBody::Generic {
                identifier,
                constraints,
            } => {
                write!(f, "{}", identifier)?;
                if !constraints.is_empty() {
                    write!(f, ": ")?;
                    for (i, constraint) in constraints.iter().enumerate() {
                        write!(f, "{:?}", constraint)?;
                        if i != constraints.len() - 1 {
                            write!(f, ", ")?;
                        }
                    }
                }
            }
            TypeBody::Array {
                type_,
                size,
                refs,
                nullable,
            } => {
                write!(f, "[{:?}; {}]", type_, size)?;
                if *nullable {
                    write!(f, "?")?;
                }
                write!(f, " refs: {}", refs)?;
            }
            TypeBody::Void => {
                write!(f, "void")?;
            }
        }
        Ok(())
    }
}

impl Kind {
    pub fn new(body: TypeBody, line: Line, file: Option<String>) -> Self {
        Kind { body, line, file }
    }

    pub fn from_struct(ident: String, file: String, line: Line) -> Self {
        Kind {
            body: TypeBody::Type {
                refs: 0,
                main: vec![ident],
                generics: Vec::new(),
                nullable: false,
                kind: KindType::Struct,
            },
            line,
            file: Some(file),
        }
    }

    pub fn from_userdata(ident: String, file: String, line: Line) -> Self {
        Kind {
            body: TypeBody::Type {
                refs: 0,
                main: vec![ident],
                generics: Vec::new(),
                nullable: false,
                kind: KindType::UserData,
            },
            line,
            file: Some(file),
        }
    }

    pub fn from_enum(enum_: &Enum, file: String) -> Self {
        Kind {
            body: TypeBody::Type {
                refs: 0,
                main: vec![enum_.identifier.clone()],
                generics: Vec::new(),
                nullable: false,
                kind: KindType::Enum,
            },
            line: enum_.line,
            file: Some(file),
        }
    }

    pub fn from_trait(trait_: &Trait, file: String) -> Self {
        Kind {
            body: TypeBody::Type {
                refs: 0,
                main: vec![trait_.identifier.clone()],
                generics: Vec::new(),
                nullable: false,
                kind: KindType::Trait,
            },
            line: trait_.line,
            file: Some(file),
        }
    }

    pub fn from_fun(fun: &Function, file: String) -> Self {
        let main = vec![fun.identifier.as_ref().unwrap().clone()];
        Kind {
            body: TypeBody::Function {
                args: fun.args.clone(),
                return_type: Box::new(fun.return_type.clone()),
                refs: 0,
            },
            line: fun.line,
            file: Some(file),
        }
    }

    pub fn bfrom_fun(fun: &libloader::Function, file: String, line: Line) -> Self {
        let main = vec![fun.name.clone()];
        Kind {
            body: TypeBody::Function {
                args: fun.args.iter().map(|arg| Arg::from_barg(arg)).collect(),
                return_type: Box::new(Some(fun.return_type.clone())),
                refs: 0,
            },
            line,
            file: Some(file),
        }
    }

    pub fn from_generic(
        ident: String,
        constraints: Vec<InnerPath>,
        line: Line,
        file: Option<String>,
    ) -> Self {
        Kind {
            body: TypeBody::Generic {
                identifier: ident,
                constraints,
            },
            line,
            file,
        }
    }

    pub fn from_array(
        type_: Kind,
        size: usize,
        refs: usize,
        nullable: bool,
        line: Line,
        file: Option<String>,
    ) -> Self {
        Kind {
            body: TypeBody::Array {
                type_: Box::new(type_),
                size,
                refs,
                nullable,
            },
            line,
            file,
        }
    }

    pub fn from_type(type_: ShallowType, line: Line, file: Option<String>) -> Self {
        Kind {
            body: TypeBody::Type {
                refs: type_.refs,
                main: type_.main,
                generics: type_.generics,
                nullable: type_.nullable,
                kind: type_.kind,
            },
            line,
            file,
        }
    }

    pub fn void() -> Self {
        Kind {
            body: TypeBody::Void,
            line: Line { line: 0, column: 0 },
            file: None,
        }
    }

    pub fn is_fun(&self) -> bool {
        match &self.body {
            TypeBody::Function { .. } => true,
            _ => false,
        }
    }

    pub fn is_type(&self) -> bool {
        match &self.body {
            TypeBody::Type { .. } => true,
            _ => false,
        }
    }

    pub fn is_generic(&self) -> bool {
        match &self.body {
            TypeBody::Generic { .. } => true,
            _ => false,
        }
    }

    pub fn is_array(&self) -> bool {
        match &self.body {
            TypeBody::Array { .. } => true,
            _ => false,
        }
    }

    pub fn is_void(&self) -> bool {
        match &self.body {
            TypeBody::Void => true,
            _ => false,
        }
    }

    pub fn is_null(&self) -> bool {
        match &self.body {
            TypeBody::Type { main, refs, .. } => main.last().unwrap() == "null" && *refs == 0,
            _ => false,
        }
    }

    pub fn get_type_kind(&self) -> Option<KindType> {
        match &self.body {
            TypeBody::Type { kind, .. } => Some(*kind),
            _ => None,
        }
    }

    pub fn cmp(&self, other: &Self) -> TypeComparison {
        if self.body != other.body {
            return TypeComparison::NotEqual;
        }
        match self.body {
            TypeBody::Function {
                args,
                return_type,
                refs,
            } => {
                todo!()
            }
            TypeBody::Type {
                refs,
                main,
                generics,
                nullable,
                kind,
            } => {
                if let TypeBody::Type {
                    refs: refs2,
                    main: main2,
                    generics: generics2,
                    nullable: nullable2,
                    kind: kind2,
                } = &other.body
                {
                    if self.is_null() && other.is_null() {
                        return TypeComparison::Equal;
                    }
                    if nullable && other.is_null() {
                        return TypeComparison::Equal;
                    }
                    if !nullable && *nullable2 {
                        return TypeComparison::NotNullable;
                    }
                    if refs != *refs2 {
                        return TypeComparison::ReferenceDiff(refs as i32 - *refs2 as i32);
                    }
                    if main.last() != main2.last() {
                        return TypeComparison::NotEqual;
                    }
                    // probably not needed
                    /*if generics.len() != generics2.len() {
                        return TypeComparison::NotEqual;
                    }*/
                    /*for (i, gen) in generics.iter().enumerate() {
                        if gen != &generics2[i] {
                            return TypeComparison::NotEqual;
                        }
                    }*/
                    TypeComparison::Equal
                } else {
                    TypeComparison::NotEqual
                }
            }
            TypeBody::Array {
                type_,
                size,
                refs,
                nullable,
            } => {
                if let TypeBody::Array {
                    type_: type_2,
                    size: size_2,
                    refs: refs_2,
                    nullable: nullable_2,
                } = &other.body
                {
                    if self.is_null() && other.is_null() {
                        return TypeComparison::Equal;
                    }
                    if nullable && other.is_null() {
                        return TypeComparison::Equal;
                    }
                    if !nullable && *nullable_2 {
                        return TypeComparison::NotNullable;
                    }
                    if refs != *refs_2 {
                        return TypeComparison::ReferenceDiff(refs as i32 - *refs_2 as i32);
                    }
                    // since array size is not known at compile time, we cant compare it
                    /*if size != *size_2 {
                        return TypeComparison::ArrayDiff(size, *size_2);
                    }*/
                    type_.cmp(type_2)
                } else {
                    TypeComparison::NotEqual
                }
            }
            TypeBody::Void => TypeComparison::Equal,
            TypeBody::Generic {
                identifier,
                constraints,
            } => todo!(),
        }
    }
}
