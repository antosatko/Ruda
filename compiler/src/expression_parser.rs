use core::panic;
use std::fmt::{self, format};
use std::ops::Index;

use crate::intermediate::dictionary::*;
use crate::intermediate::AnalyzationError::ErrType;
use crate::lexer::tokenizer::{Operators, Tokens};
use crate::tree_walker::tree_walker::{Line, Node};
use crate::{intermediate, lexer};
use intermediate::dictionary::*;
use intermediate::*;

// recursive function that traverses the tree and prints it
pub fn traverse_da_fokin_value(val: &ValueType, depth: usize) {
    match val {
        ValueType::Value(val) => {
            println!("{}Value {:?}", "-".repeat(depth), val.root);
        }
        ValueType::Expression(val) => {
            println!(
                "{}Expression {:?}",
                "-".repeat(depth),
                val.operator.as_ref().unwrap()
            );
            if let Some(val) = &val.left {
                traverse_da_fokin_value(val, depth + 1);
            }
            if let Some(val) = &val.right {
                traverse_da_fokin_value(val, depth + 1);
            }
        }
        ValueType::Operator(val, _) => {
            println!("{}Operator", "-".repeat(depth));
        }
        ValueType::AnonymousFunction(val) => {
            println!("{}AnonymousFunction", "-".repeat(depth));
        }
        ValueType::Blank => {
            println!("{}Blank", "-".repeat(depth));
        }
    }
}

pub fn expr_into_tree(node: &Node, errors: &mut Vec<ErrType>, file_name: &str) -> ValueType {
    //println!("expr_into_tree: {:?}", node);
    let nodes = step_inside_arr(&node, "nodes");
    if nodes.len() == 0 {
        return ValueType::Blank;
    }
    if let Tokens::Text(str) = &nodes[0].name {
        if str == "anonymous_function" {
            return ValueType::AnonymousFunction(get_fun_siginifier(&nodes[0], errors, file_name));
        }
    }
    let mut transform = transform_expr(&nodes, errors, file_name);
    let res = list_into_tree(&mut transform);
    if let Ok(mut val) = res {
        match &mut val {
            ValueType::Expression(val) => {
                val.is_root = true;
            }
            _ => {}
        }
        return val;
    } else {
        println!("error occured while parsing expression: {:?}", res);
        unreachable!("Blank expression parse error")
    }
}

/// %/*-+<>==!=<=>=&&||
const ORDER_OF_OPERATIONS: [Operators; 13] = [
    Operators::Or,
    Operators::And,
    Operators::MoreEq,
    Operators::LessEq,
    Operators::NotEqual,
    Operators::DoubleEq,
    Operators::AngleBracket(true),
    Operators::AngleBracket(false),
    Operators::Minus,
    Operators::Plus,
    Operators::Star,
    Operators::Slash,
    Operators::Mod,
];

/// recursive function that transforms list of values into tree
pub fn list_into_tree(list: &mut Vec<ValueType>) -> Result<ValueType, TreeTransformError> {
    let mut result = ExprNode::blank();
    if list.len() == 0 {
        return Ok(ValueType::Blank);
    }
    if list.len() == 1 {
        if let ValueType::Operator(_, line) = &list[0] {
            return Err(TreeTransformError::ExcessOperator(line.clone()));
        }
        return Ok(list.pop().unwrap());
    }
    for op in &ORDER_OF_OPERATIONS {
        let mut i = list.len() - 1;
        // if the list consists of only 1 value and it is not an operator, return it
        while i > 0 {
            if let ValueType::Operator(op2, line) = list[i] {
                if *op == op2 {
                    if i == 0 {
                        return Err(TreeTransformError::NoValue(line.clone()));
                    }
                    if i == list.len() - 1 {
                        return Err(TreeTransformError::ExcessOperator(line.clone()));
                    }
                    result.operator = Some(*op);
                    // split list into 2 lists using index
                    list.remove(i);
                    let mut right = list.split_off(i);
                    // call this function recursively for each side
                    // left side
                    if list.len() == 0 {
                        return Err(TreeTransformError::NoValue(line.clone()));
                    }
                    let res = list_into_tree(list);
                    if let Ok(left) = res {
                        result.left = Some(left);
                    } else {
                        return res;
                    }
                    // right side
                    if right.len() == 0 {
                        return Err(TreeTransformError::NoValue(line.clone()));
                    }
                    let res = list_into_tree(&mut right);
                    result.line = line;
                    if let Ok(right) = res {
                        result.right = Some(right);
                    } else {
                        return res;
                    }

                    // return result
                    return Ok(ValueType::Expression(Box::new(result)));
                }
            }
            i -= 1;
        }
    }
    return Err(TreeTransformError::NotImplementedCuzLazy);
}

#[derive(Debug)]
pub enum TreeTransformError {
    NoValue(Line),
    ExcessOperator(Line),
    ExcessValue(Line),
    NotImplementedCuzLazy,
}

impl std::fmt::Display for TreeTransformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TreeTransformError::NoValue(line) => {
                write!(f, "No value for operator at line {}", line)
            }
            TreeTransformError::ExcessOperator(line) => {
                write!(f, "Excess operator at line {}", line)
            }
            TreeTransformError::ExcessValue(line) => write!(f, "Excess value at line {}", line),
            TreeTransformError::NotImplementedCuzLazy => write!(f, "Not implemented cuz lazy"),
        }
    }
}

pub fn transform_expr(
    nodes: &Vec<Node>,
    errors: &mut Vec<ErrType>,
    file_name: &str,
) -> Vec<ValueType> {
    let mut result = vec![];
    for node in nodes {
        if let Some(op) = try_get_op(&node, errors) {
            result.push(ValueType::Operator(op, node.line));
            continue;
        }
        if let Some(val) = try_get_value(&node, errors, file_name) {
            result.push(val);
            continue;
        }
    }
    result
}

pub fn try_get_value(node: &Node, errors: &mut Vec<ErrType>, file_name: &str) -> Option<ValueType> {
    if let Tokens::Text(txt) = &node.name {
        if txt != "value" {
            return None;
        }
    }
    let prepend = get_prepend(step_inside_val(&node, "prepend"), errors);
    let tail = get_tail(step_inside_val(&node, "tail"), errors, file_name);
    let root = get_root(step_inside_val(&node, "root"), errors, file_name);
    return Some(ValueType::Value(Variable {
        unary: prepend.2,
        refs: prepend.0,
        modificatior: prepend.1,
        // change this to real root
        root: (root, step_inside_val(&node, "root").line),
        tail,
        line: node.line,
    }));
}

/// returns root of value
fn get_root(_node: &Node, errors: &mut Vec<ErrType>, file_name: &str) -> Root {
    if let Tokens::Text(txt) = &_node.name {
        if txt != "value_root" {
            unreachable!("get_root has to be called on root node, please report this bug")
        }
    }
    let node = step_inside_val(&_node, "value");
    match &node.name {
        Tokens::Text(txt) => {
            if txt == "free_parenthesis" {
                let expr = try_get_parenthesis(&node, errors, file_name);
                if let Some(expr) = expr {
                    return Root::Parenthesis(Box::new(expr));
                }
            }
            if txt == "ident" {
                let ident = match &step_inside_val(&node, "identifier").name {
                    Tokens::Text(txt) => txt,
                    _ => unreachable!("ident has to have identifier as a child, please report this bug"),
                };
                return Root::Identifier(ident.clone());
            }
            if txt == "array_expr" {
                let array = step_inside_val(&node, "array");
                let name = if let Tokens::Text(txt) = &array.name {
                    txt.as_str()
                } else {
                    unreachable!("array_expr has to have array as a child, please report this bug")
                };
                match name {
                    "array_builder" => {
                        let value = expr_into_tree(&step_inside_val(&array, "value"), errors, file_name);
                        let size = expr_into_tree(&step_inside_val(&array, "size"), errors, file_name);
                        return Root::Literal(Literals::Array(ArrayRule::Fill {
                            value: Box::new(value),
                            size: Box::new(size),
                        }));
                    }
                    "array_literal" => {
                        let values = step_inside_arr(&array, "values");
                        let mut result = vec![];
                        for value in values {
                            result.push(expr_into_tree(&value, errors, file_name));
                        }
                        return Root::Literal(Literals::Array(ArrayRule::Explicit(result)));
                    }
                    _ => unreachable!("array_expr has to be either array_builder or array_literal, please report this bug")
                }
            }
        }
        Tokens::Number(_, _) => {
            return Root::Literal(Literals::Number(node.name.clone()));
        }
        Tokens::String(str) => {
            return Root::Literal(Literals::String(str.clone()));
        }
        Tokens::Char(chr) => {
            return Root::Literal(Literals::Char(chr.clone()));
        }
        _ => {
            unreachable!("get_root has to be called on root node, please report this bug")
        }
    }
    unreachable!("get_root has to be called on root node, please report this bug")
}

pub fn try_get_literal(
    node: &Node,
    errors: &mut Vec<ErrType>,
    prepend: &(Ref, Option<(String, Line)>, Vec<(Operators, Line)>),
    file_name: &str,
) -> Option<Literal> {
    if let Tokens::Text(txt) = &node.name {
        if txt != "literal" {
            return None;
        }
    }
    let this = step_inside_val(&node, "value");
    if let Tokens::Number(_, _) = &this.name {
        return Some(Literal {
            unary: prepend.2.clone(),
            refs: prepend.0.clone(),
            modificatior: prepend.1.clone(),
            value: Literals::Number(this.name.clone()),
            line: this.line,
        });
    }
    if let Tokens::String(str) = &this.name {
        return Some(Literal {
            unary: prepend.2.clone(),
            refs: prepend.0.clone(),
            modificatior: prepend.1.clone(),
            value: Literals::String(str.clone()),
            line: this.line,
        });
    }
    if let Tokens::Char(chr) = &this.name {
        return Some(Literal {
            unary: prepend.2.clone(),
            refs: prepend.0.clone(),
            modificatior: prepend.1.clone(),
            value: Literals::Char(chr.clone()),
            line: this.line,
        });
    }
    if let Tokens::Text(txt) = &this.name {
        if txt == "array_expr" {
            let array = step_inside_val(&this, "array");
            let name = if let Tokens::Text(txt) = &array.name {
                txt.as_str()
            } else {
                unreachable!("array_expr has to have array as a child, please report this bug")
            };
            match name {
                "array_builder" => {
                    let value = expr_into_tree(&step_inside_val(&array, "value"), errors, file_name);
                    let size = expr_into_tree(&step_inside_val(&array, "size"), errors, file_name);
                    return Some(Literal {
                        unary: prepend.2.clone(),
                        refs: prepend.0.clone(),
                        modificatior: prepend.1.clone(),
                        value: Literals::Array(ArrayRule::Fill {
                            value: Box::new(value),
                            size: Box::new(size),
                        }),
                        line: array.line
                    });
                }
                "array_literal" => {
                    let values = step_inside_arr(&array, "values");
                    let mut result = vec![];
                    for value in values {
                        result.push(expr_into_tree(&value, errors, file_name));
                    }
                    return Some(Literal {
                        unary: prepend.2.clone(),
                        refs: prepend.0.clone(),
                        modificatior: prepend.1.clone(),
                        value: Literals::Array(ArrayRule::Explicit(result)),
                        line: array.line
                    });
                }
                _ => unreachable!("array_expr has to be either array_builder or array_literal, please report this bug")
            }
        }
    }
    None
}

pub fn try_get_variable(
    node: &Node,
    errors: &mut Vec<ErrType>,
    file_name: &str,
) -> Option<((String, Line), Vec<(TailNodes, Line)>)> {
    if let Tokens::Text(txt) = &node.name {
        if txt != "variable" {
            return None;
        }
    }
    let ident = get_ident(&node);
    let tail = get_tail(step_inside_val(&node, "tail"), errors, file_name);
    Some(((ident, node.line), tail))
}

pub fn get_args(node: &Node, errors: &mut Vec<ErrType>, file_name: &str) -> Vec<ValueType> {
    let mut result = vec![];
    for child in step_inside_arr(&node, "expressions") {
        let expr = expr_into_tree(&child, errors, file_name);
        if let ValueType::Expression(exrp) = &expr {
            if exrp.left.is_none() && exrp.right.is_none() {
                continue;
            }
        }
        result.push(expr);
    }
    result
}

pub fn try_get_parenthesis(
    node: &Node,
    errors: &mut Vec<ErrType>,
    file_name: &str,
) -> Option<ValueType> {
    if let Tokens::Text(txt) = &node.name {
        if txt != "free_parenthesis" {
            return None;
        }
    }
    let expression = expr_into_tree(step_inside_val(&node, "expression"), errors, file_name);
    Some(expression)
}

pub fn get_tail(node: &Node, errors: &mut Vec<ErrType>, file_name: &str) -> Vec<(TailNodes, Line)> {
    let mut tail = vec![];
    for child in step_inside_arr(&node, "nodes") {
        if let Tokens::Text(txt) = &child.name {
            if txt == "idx" {
                let expr = expr_into_tree(step_inside_val(&child, "expression"), errors, file_name);
                tail.push((TailNodes::Index(expr), child.line));
                continue;
            }
            else if txt == "nested" {
                tail.push((TailNodes::Nested(get_ident(&child)), child.line));
                continue;
            }
            else if txt == "function_call" {
                let generic = get_generics_expr(&child, errors, file_name);
                let args = get_args(step_inside_val(&child, "parenthesis"), errors, file_name);
                tail.push((TailNodes::Call(FunctionCall { generic, args }), child.line));
                continue;
            }
            else if txt == "cast" {
                let kind = get_type(step_inside_val(&child, "type"), errors, file_name);
                tail.push((TailNodes::Cast(kind), child.line));
                break;
            }
        }else if let Tokens::Optional = &child.name {
            tail.push((TailNodes::Nullable, child.line));
            continue;
        }
    }
    tail
}

pub fn get_prepend(
    node: &Node,
    _errors: &mut Vec<ErrType>,
) -> (Ref, Option<(String, Line)>, Vec<(Operators, Line)>) {
    // TODO: use "ref_tok" instead of "ref_type"
    let refs = get_ref_type(&node);
    let temp = &step_inside_val(&node, "keywords");
    let modificator = if let Tokens::Text(txt) = &temp.name {
        if txt == "'none" {
            None
        } else {
            let name = if let Tokens::Text(txt) = &step_inside_val(&temp, "kw").name {
                txt.to_string()
            } else {
                unreachable!("value_mod has to be a text token, please report this bug")
            };
            Some((name, temp.line))
        }
    } else {
        None
    };
    let mut unaries = Vec::new();
    if let Some(un) = try_step_inside_arr(&step_inside_val(&node, "unary"), "op") {
        for op_node in un {
            if let Tokens::Operator(op) = op_node.name {
                unaries.push((op, op_node.line));
            }
        }
    }
    (refs, modificator, unaries)
}

pub fn get_ref_type(node: &Node) -> Ref {
    let mut ampersants = 0;
    let mut stars = 0;
    let node = step_inside_val(&node, "ref");
    for tok in step_inside_arr(&node, "tokens") {
        match tok.name {
            Tokens::Operator(Operators::Ampersant) => ampersants += 1,
            Tokens::Operator(Operators::Star) => stars += 1,
            Tokens::Operator(Operators::And) => ampersants += 2,
            _ => {}
        }
    }
    if ampersants == stars {
        return Ref::None;
    }
    // find out if it's a reference or a dereference, if both, then subtract lesser from greater
    if ampersants > stars {
        Ref::Reference(ampersants - stars)
    } else {
        Ref::Dereferencing(stars - ampersants)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ref {
    Dereferencing(usize),
    Reference(usize),
    None,
}

pub fn try_get_op(node: &Node, _errors: &mut Vec<ErrType>) -> Option<Operators> {
    if let Tokens::Text(txt) = &node.name {
        if txt != "operator" {
            return None;
        }
    }
    if let Tokens::Operator(op) = step_inside_val(&node, "op").name {
        return Some(op);
    }
    None
}

#[derive(Debug, Clone)]
pub struct ExprNode {
    pub left: Option<ValueType>,
    pub right: Option<ValueType>,
    pub operator: Option<Operators>,
    pub line: Line,
    pub is_root: bool,
}
impl ExprNode {
    pub fn new(
        left: Option<ValueType>,
        right: Option<ValueType>,
        operator: Option<Operators>,
        line: Line,
        is_root: bool,
    ) -> ExprNode {
        ExprNode {
            left,
            right,
            operator,
            line,
            is_root,
        }
    }
    pub fn blank() -> ExprNode {
        ExprNode {
            left: None,
            right: None,
            operator: None,
            line: Line::from((0, 0)),
            is_root: false,
        }
    }
}
#[derive(Debug, Clone)]
pub enum ValueType {
    AnonymousFunction(Function),
    // parenthesis moved as a root of value
    // Parenthesis(Box<ValueType>, Vec<(TailNodes, Line)>, Vec<(Operators, Line)>, Option<(String, Line)>),
    Expression(Box<ExprNode>),
    /// only for inner functionality
    Operator(Operators, Line),
    Value(Variable),
    Blank,
}
impl ValueType {
    pub fn fun(fun: Function) -> ValueType {
        ValueType::AnonymousFunction(fun)
    }
    pub fn is_expression(&self) -> bool {
        match self {
            ValueType::Expression(_) => true,
            _ => false,
        }
    }
    pub fn get_line(&self) -> Line {
        match self {
            ValueType::Expression(expr) => expr.line,
            ValueType::Operator(_, line) => line.clone(),
            ValueType::Value(val) => val.line,
            ValueType::AnonymousFunction(fun) => fun.line,
            ValueType::Blank => Line::from((0, 0)),
        }
    }
}
#[derive(Debug, Clone)]
pub struct Literal {
    pub unary: Vec<(Operators, Line)>,
    pub refs: Ref,
    /// atm only keyword new, so bool would be sufficient, but who knows what will be in the future updates
    pub modificatior: Option<(String, Line)>,
    pub value: Literals,
    pub line: Line,
}
impl Literal {
    pub fn is_simple(&self) -> bool {
        self.refs == Ref::None && self.modificatior.is_none()
    }
}
#[derive(Debug, Clone)]
pub enum Literals {
    Number(Tokens),
    Array(ArrayRule),
    String(String),
    Char(char),
}
#[derive(Clone)]
pub enum ArrayRule {
    Fill {
        value: Box<ValueType>,
        size: Box<ValueType>,
    },
    Explicit(Vec<ValueType>),
}

impl fmt::Debug for ArrayRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArrayRule::Fill { .. } => write!(f, "[_; _]"),
            ArrayRule::Explicit(vec) => {
                write!(f, "[{}]", vec.len())
            }
        }
    }
}
#[derive(Debug, Clone)]
pub struct Variable {
    pub unary: Vec<(Operators, Line)>,
    pub refs: Ref,
    /// atm only keyword new, so bool would be sufficient, but who knows what will be in the future updates
    pub modificatior: Option<(String, Line)>,
    /// for longer variables
    /// example: *danda[5].touch_grass(9)
    ///           ~~~~~ <- this is considered root
    pub root: (Root, Line),
    /// for longer variables
    /// example: danda[5].touch_grass(9)
    /// danda is root .. rest is tail
    pub tail: Vec<(TailNodes, Line)>,
    pub line: Line,
}

#[derive(Debug, Clone)]
pub enum Root {
    Literal(Literals),
    Parenthesis(Box<ValueType>),
    Identifier(String),
}

impl Variable {
    /// returns true if the variable is simple
    pub fn is_simple(&self) -> bool {
        self.refs == Ref::None
            && self.modificatior.is_none()
            && self.tail.iter().all(|x| match x.0 {
                TailNodes::Nested(_) => true,
                _ => false,
            })
    }
    pub fn is_true_simple(&self) -> bool {
        self.refs == Ref::None
            && self.modificatior.is_none()
            && self.tail.len() == 0
            && self.unary.len() == 0
    }
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub generic: Vec<Kind>,
    pub args: Vec<ValueType>,
}
#[derive(Debug, Clone)]
pub enum TailNodes {
    Nested(String),
    Index(ValueType),
    Call(FunctionCall),
    Nullable,
    Cast(Kind),
}
