pub mod dictionary {
    use crate::{
        codeblock_parser,
        expression_parser::{self, get_args, ValueType},
        lexer::tokenizer::{Operators, Tokens},
        libloader,
        tree_walker::tree_walker::{self, ArgNodeType, Err, Node, Line},
    };
    use core::panic;
    use std::{collections::HashMap, fs::DirEntry, io::Read};

    use super::AnalyzationError::{self, ErrType};

    pub fn from_ast(ast: &HashMap<String, tree_walker::ArgNodeType>, globals: &Vec<String>) -> (Dictionary, Vec<ErrType>) {
        let mut global_dict = Dictionary::new();
        let mut errors = Vec::new();
        if let Some(ArgNodeType::Array(entry)) = ast.get("nodes") {
            load_dictionary(entry, &mut global_dict, &mut errors)
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
        println!("final");
        for i in 0..dictionary.constants.len() {
            let constant = &dictionary.constants[i];
            if let Some(val) = analyze_const(&constant.value, dictionary, errors) {
                changes.push((i, val));
            }
        }
        if changes.len() > 0 {
            while let Some((i, val)) = changes.pop() {
                println!("{} {:?}", i, val);
                dictionary.constants[i].real_value = Some(val);
            }
        } else {
            // look if every constant has a value
            for constant in &dictionary.constants {
                if constant.real_value.is_none() {
                    println!("2: {} {:?}", constant.identifier, constant.value);
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
            ValueType::Literal(val) => {
                if val.is_simple() {
                    ConstValue::from_literal(&val, &dictionary, errors)
                } else {
                    None
                }
            }
            ValueType::AnonymousFunction(fun) => Some(ConstValue::Function((*fun).clone())),
            ValueType::Value(val) => {
                let unary = if let Some(unary) = &val.unary {
                    *unary
                } else {
                    (Operators::DoubleEq, val.line)
                };
                let val = if val.is_simple() {
                    if val.tail.len() == 0 {
                        match val.root.0.as_str() {
                            "true" => Some(ConstValue::Bool(true)),
                            "false" => Some(ConstValue::Bool(false)),
                            _ => {
                                let const_val = dictionary.find_const(&val.root.0);
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
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };
                match val {
                    Some(mut val) => {
                        val.apply_unary(&unary.0);
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
                    return None
                };
                if right.is_none() {
                    return None
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
            ValueType::Parenthesis(v, t) => {
                if t.len() != 0 {
                    None
                } else {
                    analyze_const(v, dictionary, errors)
                }
            }
            _ => None,
        }
    }
    pub fn load_dictionary(
        nodes: &Vec<Node>,
        dictionary: &mut Dictionary,
        errors: &mut Vec<ErrType>,
    ) {
        for node in nodes {
            load_node(node, dictionary, errors);
        }
    }
    pub fn load_node(node: &Node, dictionary: &mut Dictionary, errors: &mut Vec<ErrType>) {
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
                    line: node.line
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
                            errors
                                .push(ErrType::EnumVariantAssignedIdent(ident.to_string(), enum_value.line))
                        }
                    }
                    result.keys.push((ident, n, enum_value.line));
                }
                if dictionary.register_id(result.identifier.to_string(), IdentifierKinds::Enum) {
                    dictionary.enums.push(result);
                } else {
                    errors.push(ErrType::ConflictingNames(result.identifier.to_string(), node.line))
                }
            }
            "KWType" => {
                let name = get_ident(&node);
                if dictionary.register_id(name.to_string(), IdentifierKinds::Type) {
                    dictionary.types.push(TypeDef {
                        kind: get_type(step_inside_val(&node, "type"), errors),
                        identifier: name,
                        generics: get_generics_decl(&node, errors),
                        overloads: vec![],
                        methods: vec![],
                        public: public(&node),
                        line: node.line
                    })
                } else {
                    errors.push(ErrType::ConflictingNames(name.to_string(), node.line))
                }
            }
            "KWStruct" => {
                let mut result = Struct {
                    identifier: get_ident(node),
                    fields: Vec::new(),
                    generics: get_generics_decl(node, errors),
                    traits: Vec::new(),
                    public: public(&node),
                    memory_layout: Vec::new(),
                    ptrs: 0,
                    line: node.line
                };
                for key in step_inside_arr(node, "keys") {
                    let ident = get_ident(&key);
                    for field in &result.fields {
                        if *field.0 == ident {
                            errors.push(ErrType::StructVariantAssignedIdent(
                                ident.to_string(),
                                Line {
                                    line: 0,
                                    column: 0,
                                }
                            ))
                        }
                    }
                    result.fields.push((
                        get_ident(key),
                        get_type(step_inside_val(key, "type"), errors),
                    ))
                }
                if dictionary.register_id(result.identifier.to_string(), IdentifierKinds::Struct) {
                    dictionary.structs.push(result);
                } else {
                    errors.push(ErrType::ConflictingNames(result.identifier.to_string(), node.line))
                }
            }
            "KWImport" => {
                let alias = try_get_ident(&node);
                let path = if let Tokens::String(path) = &step_inside_val(&node, "path").name {
                    path.to_string()
                } else {
                    unreachable!("Path not specified");
                };
                dictionary.imports.push(Import {
                    path,
                    alias,
                    line: node.line
                });
            }
            "KWFun" => {
                let fun = get_fun_siginifier(&node, errors);
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
                        line: node.line
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
                        ),
                        real_value: None,
                        line: node.line
                    })
                } else {
                    errors.push(ErrType::ConflictingNames(identifier.to_string(), node.line))
                }
                //expression_parser::traverse_da_fokin_value(&expression_parser::expr_into_tree(step_inside_val(&node, "expression"), errors), 0);
                //println!("{:#?}", expression_parser::expr_into_tree(step_inside_val(&node, "expression"), errors));
            }
            "KWImpl" => {
                let ident = get_nested_ident(&step_inside_val(&node, "identifier"), errors);
                let mut functions = Vec::new();
                let mut overloads = Vec::new();
                let traits = get_traits(&node, errors);
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
                    traits,
                    functions,
                    overloads,
                    line: node.line
                })
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
                                overloads.push(get_overload_siginifier(&method, errors))
                            }
                            "KWFun" => {
                                functions.push(get_fun_siginifier(&method, errors));
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
                        line: node.line
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
                    let kind = get_type(&step_inside_val(&arg, "type"), errors);
                    args.push(Arg {
                        identifier: ident,
                        kind,
                        line: arg.line
                    })
                }
                let mut fields = Vec::new();
                for field in step_inside_arr(&node, "fields") {
                    let ident = get_ident(&field);
                    let val = step_inside_val(&step_inside_val(&field, "value"), "expression");
                    if let Tokens::Text(txt) = &val.name {
                        match txt.as_str() {
                            "expression" => {
                                let expr = expression_parser::expr_into_tree(&val, errors);
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
                        line: node.line
                    })
                } else {
                    errors.push(ErrType::ConflictingNames(ident.to_string(), node.line))
                }
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
    pub fn get_nested_ident(node: &Node, errors: &mut Vec<ErrType>) -> NestedIdent {
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
    pub fn get_overload_siginifier(node: &Node, errors: &mut Vec<ErrType>) -> Overload {
        let operator = get_operator(step_inside_val(&node, "op"));
        let generics = get_generics_decl(&node, errors);
        let kind = if let Some(kind) = try_step_inside_val(step_inside_val(&node, "type"), "type") {
            Some(get_type(kind, errors))
        } else {
            None
        };
        let arg = step_inside_val(&node, "arg");

        // fujj
        let code = if node.nodes.contains_key("code") {
            codeblock_parser::generate_tree(step_inside_val(&node, "code"), errors)
        } else {
            vec![]
        };

        Overload {
            operator,
            arg: Arg {
                identifier: get_ident(&arg),
                kind: get_type(step_inside_val(&arg, "type"), errors),
                line: arg.line
            },
            stack_size: None,
            location: None,
            return_type: kind,
            generics,
            public: public(&node),
            code,
            line: node.line
        }
    }
    pub fn get_fun_siginifier(node: &Node, errors: &mut Vec<ErrType>) -> Function {
        let identifier = if node.nodes.contains_key("identifier") {
            Some(get_ident(&node))
        } else {
            None
        };
        let generics = get_generics_decl(&node, errors);
        let kind = if let Some(kind) = try_step_inside_val(step_inside_val(&node, "type"), "type") {
            Some(get_type(kind, errors))
        } else {
            None
        };
        let mut args = Vec::new();
        for arg in step_inside_arr(node, "arguments") {
            if let Tokens::Text(name) = &arg.name {
                match name.as_str() {
                    "self_arg" => {
                        args.push(Arg{
                            identifier: String::from("self"),
                            kind: ShallowType {
                                is_fun: None,
                                arr_len: None,
                                refs: count_refs(&arg),
                                main: vec![String::from("Self")],
                                generics: Vec::new(),
                            },
                            line: arg.line
                        });
                    }
                    "arg" => {
                        let ident = get_ident(arg);
                        for arg in &args {
                            if arg.identifier == ident {
                                errors.push(ErrType::ConflictingArgsName(ident.to_string(), arg.line));
                            }
                        }
                        args.push(Arg{
                            identifier: ident,
                            kind: get_type(step_inside_val(&arg, "type"), errors),
                            line: arg.line
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
            codeblock_parser::generate_tree(step_inside_val(&node, "code"), errors)
        } else {
            vec![]
        };
        Function {
            can_yeet,
            identifier,
            args,
            stack_size: None,
            location: None,
            return_type: kind,
            generics,
            public: false,
            code,
            line: node.line
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
    pub fn get_type(node: &Node, errors: &mut Vec<ErrType>) -> ShallowType {
        let main = step_inside_val(&node, "main");
        if main.name == Tokens::Text(String::from("function_head")) {
            let fun = get_fun_siginifier(&main, errors);
            let refs = count_refs(&node);
            return ShallowType {
                is_fun: Some(Box::new(fun)),
                arr_len: None,
                refs,
                main: vec![],
                generics: Vec::new(),
            };
        }
        let mut arr_len = None;
        let mut refs = count_refs(node);
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
            let mut main = vec![];
            let arr = step_inside_val(&node, "arr");
            if let Some(arr) = try_step_inside_arr(
                step_inside_val(step_inside_val(&arr, "type"), "main"),
                "nodes",
            ) {
                for path_part in arr {
                    if let Tokens::Text(txt) = get_token(path_part, "identifier") {
                        main.push(txt.to_string())
                    }
                }
            }
            // length will be calculated later since it might be a constant or an expression with constant value
            // consts will be evaluated after the dictionary is loaded
            arr_len = Some(0);
            main
        };
        ShallowType {
            is_fun: None,
            arr_len,
            refs,
            main,
            generics: get_generics_expr(node, errors),
        }
    }
    pub fn get_generics_expr(node: &Node, errors: &mut Vec<ErrType>) -> GenericExpr {
        let mut result = Vec::new();
        if let Some(arr) = try_step_inside_arr(step_inside_val(node, "generic"), "types") {
            for generic_expr in arr {
                result.push(get_type(generic_expr, errors));
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
        pub implementations: Vec<Implementation>,
        pub traits: Vec<Trait>,
        pub errors: Vec<Error>,
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
        pub alias: Option<String>,
        pub line: Line,
    }
    #[derive(Debug)]
    pub struct Trait {
        pub identifier: String,
        pub methods: Vec<Function>,
        pub overloads: Vec<Overload>,
        // dependences
        pub traits: Vec<NestedIdent>,
        pub public: bool,
        pub line: Line
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
    #[derive(Debug)]
    pub struct TypeDef {
        pub kind: ShallowType,
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
    #[derive(Debug, Clone)]
    pub struct Arg {
        pub identifier: String,
        pub kind: ShallowType,
        pub line: Line,
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
        pub location: Option<usize>,
        pub return_type: Option<ShallowType>,
        pub can_yeet: bool,
        pub generics: Vec<GenericDecl>,
        pub public: bool,
        pub code: Vec<codeblock_parser::Nodes>,
        pub line: Line,
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
        pub return_type: Option<ShallowType>,
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
        pub fields: Vec<(String, ShallowType)>,
        pub traits: Vec<NestedIdent>,
        pub public: bool,
        pub memory_layout: Vec<(String, usize)>,
        /// number of pointers in struct
        pub ptrs: usize,
        pub line: Line,
    }
    #[derive(Debug)]
    pub struct Implementation {
        pub target: NestedIdent,
        pub traits: Vec<NestedIdent>,
        pub functions: Vec<Function>,
        pub overloads: Vec<Overload>,
        pub line: Line,
    }
    #[derive(Debug)]
    pub struct Variable {
        pub kind: Option<ShallowType>,
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
            literal: &expression_parser::Literal,
            dictionary: &Dictionary,
            errors: &mut Vec<ErrType>,
        ) -> Option<ConstValue> {
            let negate = if let Some((Operators::Minus, _)) = literal.unary {
                -1.0
            } else {
                1.0
            };
            if literal.is_simple() {
                match &literal.value {
                    expression_parser::Literals::Number(num) => {
                        if let Tokens::Number(num, kind) = *num {
                            match kind {
                                'f' => Some(ConstValue::Float(num as f64 * negate)),
                                'u' => Some(ConstValue::Usize(num as usize)),
                                'n' => Some(ConstValue::Number(num * negate)),
                                'i' => Some(ConstValue::Int(num as i64 * negate as i64)),
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
                    expression_parser::Literals::String(str) => {
                        Some(ConstValue::String(str.clone()))
                    }
                }
            } else {
                None
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
                _ => {},
            }
        }
    }
    /*#[derive(Debug)]
    pub enum Types {
        Int,
        Float,
        Usize,
        Char,
        Byte,
        Bool,
        Null,
        /// refference type
        Pointer(Box<Types>),
        /// type of an array, lenght
        Array(Box<Types>, usize),
        /// non-primmitive types holding their identifiers
        Function(String, GenericExpr),
        Enum(String, GenericExpr),
        Struct(String, GenericExpr),
    }*/
    type GenericExpr = Vec<ShallowType>;

    #[derive(Clone)]
    pub struct ShallowType {
        pub is_fun: Option<Box<Function>>,
        /// if Some then it is an array of that length
        pub arr_len: Option<usize>,
        pub refs: usize,
        pub main: NestedIdent,
        pub generics: GenericExpr,
    }
    // print formating
    impl std::fmt::Debug for ShallowType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
            if self.arr_len.is_some() {
                write!(f, "[")?;
            }
            for (i, part) in self.main.iter().enumerate() {
                write!(f, "{}", part)?;
                if i != self.main.len() - 1 {
                    write!(f, ".")?;
                }
            }
            if self.arr_len.is_some() {
                write!(f, "; {}]", self.arr_len.unwrap())?;
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
            Ok(())
        }
    }
    impl ShallowType {
        pub fn empty() -> Self {
            ShallowType {
                is_fun: None,
                arr_len: None,
                refs: 0,
                main: vec![],
                generics: vec![],
            }
        }
        pub fn cmp(&self, other: &Self, dict: &Dictionary) -> TypeComparison {
            /*// check if both have the same refs and if not return difference in refs
            if self.refs != other.refs {
                return TypeComparison::ReferenceDiff(self.refs as i32 - other.refs as i32);
            }
            // check if one of them as an array and if so return difference in array length
            if self.arr_len.is_some() || other.arr_len.is_some() {
                if self.arr_len.is_none() {
                    return TypeComparison::NotEqual;
                }
                if other.arr_len.is_none() {
                    return TypeComparison::NotEqual;
                }
                return TypeComparison::ArrayDiff(self.arr_len.unwrap(), other.arr_len.unwrap());
            }
            if self.main != other.main {
                return TypeComparison::Different;
            }
            if self.generics.len() != other.generics.len() {
                return TypeComparison::Different;
            }
            for (i, gen) in self.generics.iter().enumerate() {
                if gen != &other.generics[i] {
                    return TypeComparison::Different;
                }
            }*/
            TypeComparison::Equal
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
                implementations: vec![],
                traits: vec![],
                errors: vec![],
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
        InvalidConstant(crate::lexer::tokenizer::Tokens),
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
}
