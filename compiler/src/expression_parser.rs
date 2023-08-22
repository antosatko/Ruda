use core::panic;
use std::fmt::{format, self};
use std::ops::Index;

use crate::intermediate::dictionary::*;
use crate::intermediate::AnalyzationError::ErrType;
use crate::lexer::tokenizer::{Operators, Tokens};
use crate::tree_walker::tree_walker::Node;
use crate::{intermediate, lexer};
use intermediate::dictionary::*;
use intermediate::*;


// recursive function that traverses the tree and prints it
pub fn traverse_da_fokin_value(val: &ValueType, depth: usize) {
    match val {
        ValueType::Value(val) => {
            println!("{}Value {:?}", "-".repeat(depth), val.root);
        }
        ValueType::Literal(val) => {
            println!("{}Literal {:?}", "-".repeat(depth), val.value);
        }
        ValueType::Parenthesis(val, _) => {
            println!("{}Parenthesis", "-".repeat(depth));
            traverse_da_fokin_value(val, depth + 1);
        }
        ValueType::Expression(val) => {
            println!("{}Expression {:?}", "-".repeat(depth), val.operator.as_ref().unwrap());
            if let Some(val) = &val.left {
                traverse_da_fokin_value(val, depth + 1);
            }
            if let Some(val) = &val.right {
                traverse_da_fokin_value(val, depth + 1);
            }
        }
        ValueType::Operator(val) => {
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

pub fn expr_into_tree(node: &Node, errors: &mut Vec<ErrType>) -> ValueType {
    //println!("expr_into_tree: {:?}", node);
    let nodes = step_inside_arr(&node, "nodes");
    if nodes.len() == 0 {
        return ValueType::Expression(Box::new(ExprNode::blank()));
    }
    if let Tokens::Text(str) = &nodes[0].name {
        if str == "anonymous_function" {
            return ValueType::AnonymousFunction(get_fun_siginifier(&nodes[0], errors));
        }
    }
    let mut transform = transform_expr(&nodes, errors);
    let res = list_into_tree(&mut transform);
    if let Ok(val) = res {
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
    let mut result = ExprNode {
        left: None,
        right: None,
        operator: None,
    };
    if list.len() == 0 {
        return Ok(ValueType::Blank);
    }
    if list.len() == 1 {
        if let ValueType::Operator(_) = &list[0] {
            return Err(TreeTransformError::ExcessOperator);
        }
        return Ok(list.pop().unwrap());
    }
    for op in &ORDER_OF_OPERATIONS {
        let mut i = 0;
        // if the list consists of only 1 value and it is not an operator, return it
        while i < list.len() {
            if let ValueType::Operator(op2) = &list[i] {
                if op == op2 {
                    if i == 0 {
                        return Err(TreeTransformError::NoValue);
                    }
                    if i == list.len() - 1 {
                        return Err(TreeTransformError::ExcessOperator);
                    }
                    result.operator = Some(*op);
                    // split list into 2 lists using index
                    list.remove(i);
                    let mut right = list.split_off(i);
                    // call this function recursively for each side
                    // left side
                    if list.len() == 0 {
                        return Err(TreeTransformError::NoValue);
                    }
                    let res = list_into_tree(list);
                    if let Ok(left) = res {
                        result.left = Some(left);
                    } else {
                        return res;
                    }
                    // right side
                    if right.len() == 0 {
                        return Err(TreeTransformError::NoValue);
                    }
                    let res = list_into_tree(&mut right);
                    if let Ok(right) = res {
                        result.right = Some(right);
                    } else {
                        return res;
                    }

                    // return result
                    return Ok(ValueType::Expression(Box::new(result)));
                }
            }
            i += 1;
        }
    }
    return Err(TreeTransformError::NotImplementedCuzLazy);
}

#[derive(Debug)]
pub enum TreeTransformError {
    NoValue,
    ExcessOperator,
    ExcessValue,
    NotImplementedCuzLazy,
}

pub fn transform_expr(nodes: &Vec<Node>, errors: &mut Vec<ErrType>) -> Vec<ValueType> {
    let mut result = vec![];
    for node in nodes {
        if let Some(op) = try_get_op(&node, errors) {
            result.push(ValueType::Operator(op));
            continue;
        }
        if let Some(val) = try_get_value(&node, errors) {
            result.push(val);
            continue;
        }
    }
    result
}

pub fn try_get_value(node: &Node, errors: &mut Vec<ErrType>) -> Option<ValueType> {
    if let Tokens::Text(txt) = &node.name {
        if txt != "value" {
            return None;
        }
    }
    let prepend = get_prepend(step_inside_val(&node, "prepend"), errors);
    if let Some(car) = try_get_variable(step_inside_val(&node, "value"), errors) {
        return Some(ValueType::Value(Variable {
            unary: prepend.2,
            refs: prepend.0,
            modificatior: prepend.1,
            root: car.0,
            tail: car.1,
        }));
    }
    if let Some(lit) = try_get_literal(step_inside_val(&node, "value"), errors, &prepend) {
        return Some(ValueType::Literal(lit));
    }
    if let Some(paren) = try_get_parenthesis(step_inside_val(&node, "value"), errors) {
        return Some(ValueType::Parenthesis(Box::new(paren.0), paren.1));
    }
    None
}

pub fn try_get_literal(
    node: &Node,
    errors: &mut Vec<ErrType>,
    prepend: &(Ref, Option<String>, Option<Operators>),
) -> Option<Literal> {
    if let Tokens::Text(txt) = &node.name {
        if txt != "literal" {
            return None;
        }
    }
    let this = step_inside_val(&node, "value");
    if let Tokens::Number(_, _) = &this.name {
        return Some(Literal {
            unary: prepend.2,
            refs: prepend.0.clone(),
            modificatior: prepend.1.clone(),
            value: Literals::Number(this.name.clone()),
        });
    }
    if let Tokens::String(str) = &this.name {
        return Some(Literal {
            unary: prepend.2,
            refs: prepend.0.clone(),
            modificatior: prepend.1.clone(),
            value: Literals::String(str.clone()),
        });
    }
    if let Tokens::Char(chr) = &this.name {
        return Some(Literal {
            unary: prepend.2,
            refs: prepend.0.clone(),
            modificatior: prepend.1.clone(),
            value: Literals::Char(chr.clone()),
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
                    let value = expr_into_tree(&step_inside_val(&array, "value"), errors);
                    let size = expr_into_tree(&step_inside_val(&array, "size"), errors);
                    return Some(Literal {
                        unary: prepend.2,
                        refs: prepend.0.clone(),
                        modificatior: prepend.1.clone(),
                        value: Literals::Array(ArrayRule::Fill {
                            value: Box::new(value),
                            size: Box::new(size),
                        })
                    });
                }
                "array_literal" => {
                    let values = step_inside_arr(&array, "values");
                    let mut result = vec![];
                    for value in values {
                        result.push(expr_into_tree(&value, errors));
                    }
                    return Some(Literal {
                        unary: prepend.2,
                        refs: prepend.0.clone(),
                        modificatior: prepend.1.clone(),
                        value: Literals::Array(ArrayRule::Explicit(result))
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
) -> Option<(String, Vec<TailNodes>)> {
    if let Tokens::Text(txt) = &node.name {
        if txt != "variable" {
            return None;
        }
    }
    let ident = get_ident(&node);
    let tail = get_tail(step_inside_val(&node, "tail"), errors);
    Some((ident, tail))
}

pub fn get_args(node: &Node, errors: &mut Vec<ErrType>) -> Vec<ValueType> {
    let mut result = vec![];
    for child in step_inside_arr(&node, "expressions") {
        result.push(expr_into_tree(&child, errors));
    }
    result
}

pub fn try_get_parenthesis(
    node: &Node,
    errors: &mut Vec<ErrType>,
) -> Option<(ValueType, Vec<TailNodes>)> {
    if let Tokens::Text(txt) = &node.name {
        if txt != "free_parenthesis" {
            return None;
        }
    }
    let tail = get_tail(step_inside_val(&node, "tail"), errors);
    let expression = expr_into_tree(step_inside_val(&node, "expression"), errors);
    Some((expression, tail))
}

pub fn get_tail(node: &Node, errors: &mut Vec<ErrType>) -> Vec<TailNodes> {
    let mut tail = vec![];
    for child in step_inside_arr(&node, "nodes") {
        if let Tokens::Text(txt) = &child.name {
            if txt == "idx" {
                let expr = expr_into_tree(step_inside_val(&child, "expression"), errors);
                tail.push(TailNodes::Index(expr));
                continue;
            }
            if txt == "nested" {
                tail.push(TailNodes::Nested(get_ident(&child)));
                continue;
            }
            if txt == "function_call" {
                let generic = get_generics_expr(&child, errors);
                let args = get_args(step_inside_val(&child, "parenthesis"), errors);
                tail.push(TailNodes::Call(FunctionCall { generic, args }));
                continue;
            }
            if txt == "cast" {
                let kind = get_nested_ident(step_inside_val(&child, "type"), errors);
                tail.push(TailNodes::Cast(kind));
                break;
            }
        }
    }
    tail
}

pub fn get_prepend(
    node: &Node,
    errors: &mut Vec<ErrType>,
) -> (Ref, Option<String>, Option<Operators>) {
    // TODO: use "ref_tok" instead of "ref_type"
    let refs = get_ref_type(&node);
    let modificator = if let Tokens::Text(txt) = &step_inside_val(&node, "keywords").name {
        if txt == "'none" {
            None
        } else {
            Some(txt.to_string())
        }
    } else {
        None
    };
    let unary = if let Some(un) = try_step_inside_val(&step_inside_val(&node, "unary"), "op") {
        if let Tokens::Operator(op) = un.name {
            Some(op)
        } else {
            None
        }
    } else {
        None
    };
    (refs, modificator, unary)
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
    let result = if ampersants > stars {
        return Ref::Reference(ampersants - stars);
    } else {
        return Ref::Dereferencing(stars - ampersants);
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ref {
    Dereferencing(usize),
    Reference(usize),
    None,
}

pub fn try_get_op(node: &Node, errors: &mut Vec<ErrType>) -> Option<Operators> {
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
}
impl ExprNode {
    pub fn new(
        left: Option<ValueType>,
        right: Option<ValueType>,
        operator: Option<Operators>,
    ) -> ExprNode {
        ExprNode {
            left,
            right,
            operator,
        }
    }
    pub fn blank() -> ExprNode {
        ExprNode {
            left: None,
            right: None,
            operator: None,
        }
    }
}
#[derive(Debug, Clone)]
pub enum ValueType {
    Literal(Literal),
    AnonymousFunction(Function),
    /// parenthesis
    Parenthesis(Box<ValueType>, Vec<TailNodes>),
    Expression(Box<ExprNode>),
    /// only for inner functionality
    Operator(Operators),
    Value(Variable),
    Blank,
}
impl ValueType {
    pub fn fun(fun: Function) -> ValueType {
        ValueType::AnonymousFunction(fun)
    }
    pub fn value(val: Literal) -> ValueType {
        ValueType::Literal(val)
    }
}
#[derive(Debug, Clone)]
pub struct Literal {
    pub unary: Option<Operators>,
    pub refs: Ref,
    /// atm only keyword new, so bool would be sufficient, but who knows what will be in the future updates
    pub modificatior: Option<String>,
    pub value: Literals,
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
    Fill{value: Box<ValueType>, size: Box<ValueType>},
    Explicit(Vec<ValueType>),
}

impl fmt::Debug for ArrayRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArrayRule::Fill{value, size} => write!(f, "[_; _]"),
            ArrayRule::Explicit(vec) => {
                write!(f, "[{}]", vec.len())
            }
        }
    }
}
#[derive(Debug, Clone)]
pub struct Variable {
    pub unary: Option<Operators>,
    pub refs: Ref,
    /// atm only keyword new, so bool would be sufficient, but who knows what will be in the future updates
    pub modificatior: Option<String>,
    /// for longer variables
    /// example: *danda[5].touch_grass(9)
    ///           ~~~~~ <- this is considered root
    pub root: String,
    /// for longer variables
    /// example: danda[5].touch_grass(9)
    /// danda is root .. rest is tail
    pub tail: Vec<TailNodes>,
}

impl Variable {
    pub fn is_simple(&self) -> bool {
        self.refs == Ref::None && self.modificatior.is_none() && self.tail.iter().all(|x| match x {
            TailNodes::Nested(_) => true,
            _ => false
        })
    }
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    generic: Vec<ShallowType>,
    args: Vec<ValueType>,
}
#[derive(Debug, Clone)]
pub enum TailNodes {
    Nested(String),
    Index(ValueType),
    Call(FunctionCall),
    Cast(Vec<String>),
}

pub fn try_is_operator(node: &Node, errors: &mut Vec<ErrType>) -> Option<Operators> {
    if let Tokens::Operator(op) = &node.name {
        return Some(*op);
    }
    None
}
