use crate::intermediate::AnalyzationError::ErrType;
use crate::{expression_parser::*, tree_walker};
use crate::intermediate::dictionary::{ShallowType, step_inside_arr, step_inside_val, get_ident, get_type};
use crate::lexer::tokenizer::*;
use crate::type_check::*;
use crate::expression_parser::*;

pub fn generate_tree(node: &tree_walker::tree_walker::Node, errors: &mut Vec<ErrType>) -> Vec<Nodes> {
    if let Tokens::Text(txt) = &node.name {
        if txt != "code_block" {
            //errors.push(ErrType::NotCodeBlock);
            return Vec::new();
        }
    }else {
        //errors.push(ErrType::NotCodeBlock);
        return Vec::new();
    }
    let mut nodes = Vec::new();
    for node in step_inside_arr(&node, "nodes") {
        let temp = node_from_node(node, errors);
        if let Some(temp) = temp {
            nodes.push(temp);
        }
    }
    nodes
}

pub fn node_from_node(node: &tree_walker::tree_walker::Node, errors: &mut Vec<ErrType>) -> Option<Nodes> {
    if let Tokens::Text(txt) = &node.name {
        match txt.as_str() {
            "KWReturn" => {
                let expre = step_inside_val(&node, "expression");
                let expr = if step_inside_arr(&expre, "nodes").len() > 0  {
                    Some(expr_into_tree(&expre, errors))
                }else {
                    None
                };
                Some(Nodes::Return {
                    expr,
                })
            }
            "KWBreak" => Some(Nodes::Break),
            "KWContinue" => Some(Nodes::Continue),
            "KWLoop" => {
                let body = step_inside_arr(&node, "body");
                let mut nodes = Vec::new();
                for node in body {
                    let temp = node_from_node(node, errors);
                    if let Some(temp) = temp {
                        nodes.push(temp);
                    }
                }
                Some(Nodes::Loop {
                    body: nodes,
                })
            }
            "KWYeet" => {
                let expr = step_inside_val(&node, "err");
                let expr = try_get_variable(&expr, errors).unwrap();
                Some(Nodes::Yeet {
                    expr,
                })
            }
            "code_block" => {
                Some(Nodes::Block {
                    body: generate_tree(&node, errors),
                })
            }
            "set" => {
                let op = step_inside_val(&step_inside_val(&node, "operator"), "op");
                let op = if let Tokens::Operator(op) = &op.name {
                    *op
                }else {
                    errors.push(ErrType::NotOperator(node.line));
                    return None;
                };
                let target = step_inside_val(&node, "value");
                let target = try_get_value(target, errors).unwrap();
                let expr = step_inside_val(&node, "expression");
                let expr = expr_into_tree(&expr, errors);
                Some(Nodes::Set {
                    target,
                    expr,
                    op,
                })
            }
            "expression" => {
                let expr = expr_into_tree(&node, errors);
                Some(Nodes::Expr {
                    expr,
                })
            }
            "KWIf" => {
                let cond = step_inside_val(&node, "expression");
                let cond = expr_into_tree(&cond, errors);
                let body = generate_tree(step_inside_val(&node, "code"), errors);
                let mut elif = Vec::new();
                for node in step_inside_arr(&node, "elif") {
                    let cond = step_inside_val(&node, "expression");
                    let cond = expr_into_tree(&cond, errors);
                    let body = generate_tree(step_inside_val(&node, "code"), errors);
                    elif.push((cond, Nodes::Block { body }));
                }
                let els = step_inside_val(&node, "else");
                let els = if let Tokens::Text(txt) = &els.name {
                    if txt == "KWElse" {
                        Some(generate_tree(step_inside_val(&els, "code"), errors))
                    }else {
                        None
                    }
                } else {
                    None
                };

                Some(Nodes::If {
                    cond,
                    body,
                    elif,
                    els,
                })
            }
            "KWWhile" => {
                let cond = step_inside_val(&node, "expression");
                let cond = expr_into_tree(&cond, errors);
                let body = generate_tree(step_inside_val(&node, "code"), errors);
                Some(Nodes::While {
                    cond,
                    body,
                })
            }
            "KWFor" => {
                let ident = get_ident(&node);
                let expr = step_inside_val(&node, "expression");
                let expr = expr_into_tree(&expr, errors);
                let body = generate_tree(step_inside_val(&node, "code"), errors);
                Some(Nodes::For {
                    ident,
                    expr,
                    body,
                })
            }
            "KWTry" => {
                let body = generate_tree(step_inside_val(&node, "code"), errors);
                let finally = step_inside_val(&node, "finally");
                let finally = if let Tokens::Text(txt) = &finally.name {
                    if txt == "KWFinally" {
                        Some(generate_tree(step_inside_val(&finally, "code"), errors))
                    }else {
                        None
                    }
                } else {
                    None
                };
                let mut catch = Vec::new();
                for node in step_inside_arr(&node, "catch") {
                    let ident = get_ident(&node);
                    let body = generate_tree(step_inside_val(&node, "code"), errors);
                    let mut kinds = Vec::new();
                    let kinds_path = step_inside_arr(&node, "types");
                    for node in kinds_path {
                        let mut kind = Vec::new();
                        for node in step_inside_arr(&node, "nodes") {
                            let txt = if let Tokens::Text(txt) = &step_inside_val(node, "identifier").name {
                                txt
                            }else {
                                return None;
                            };
                            kind.push(txt.clone());
                        }
                        kinds.push(kind);
                    }
                    
                    catch.push(Catch { ident, kinds, body });
                }
                Some(Nodes::Try {
                    body,
                    catch,
                    finally,
                })
            }
            "KWSwitch" => {
                let expr = step_inside_val(&node, "expression");
                let expr = expr_into_tree(&expr, errors);

                let mut body = Vec::new();
                let mut default = None;
                for node in step_inside_arr(&node, "nodes") {
                    let expr = step_inside_val(&node, "expression");
                    
                    if let Tokens::Text(txt) = &step_inside_val(&expr, "ignore").name {
                        if txt == "_" {
                            default = Some(generate_tree(step_inside_val(&node, "code"), errors));
                        }else {
                            let expr = expr_into_tree(&expr, errors);
                            let bd = generate_tree(step_inside_val(&node, "code"), errors);
                            body.push((expr, bd));
                        }
                    }
                }
                return Some(Nodes::Switch {
                    expr,
                    body,
                    default
                });
            }
            "KWLet" => {
                let ident = get_ident(&node);
                let expr = step_inside_val(&node, "expression");
                let expr = if let Tokens::Text(txt) = &expr.name {
                    if txt == "expression" {
                        let expr = expr_into_tree(&expr, errors);
                        Some(expr)
                    } else {
                        None
                    }
                } else {
                    None
                };
                
                let kind = step_inside_val(&node, "type");
                let kind = if let Tokens::Text(txt) = &kind.name {
                    if txt == "type_specifier" {
                        Some(get_type(&step_inside_val(kind, "type"), errors))
                    } else {
                        None
                    }
                } else {
                    None
                };
                
                Some(Nodes::Let {
                    ident,
                    expr,
                    kind,
                })
            }

            _ => None
        }
    }else {
        None
    }
}

#[derive(Debug, Clone)]
pub enum Nodes {
    Let {
        ident: String,
        expr: Option<ValueType>,
        kind: Option<ShallowType>,
    },
    If {
        cond: ValueType,
        body: Vec<Nodes>,
        elif: Vec<(ValueType, Nodes)>,
        els: Option<Vec<Nodes>>,
    },
    While {
        cond: ValueType,
        body: Vec<Nodes>,
    },
    For {
        ident: String,
        expr: ValueType,
        body: Vec<Nodes>,
    },
    Return {
        expr: Option<ValueType>,
    },
    Expr {
        expr: ValueType,
    },
    Block {
        body: Vec<Nodes>,
    },
    Break,
    Continue,
    Loop {
        body: Vec<Nodes>,
    },
    Yeet {
        expr: (String, Vec<TailNodes>),
    },
    Try {
        body: Vec<Nodes>,
        ///     catches ((ident, [types]), body)
        catch: Vec<Catch>,
        finally: Option<Vec<Nodes>>,
    },
    Switch {
        expr: ValueType,
        body: Vec<(ValueType, Vec<Nodes>)>,
        default: Option<Vec<Nodes>>,
    },
    Set {
        target: ValueType,
        expr: ValueType,
        op: Operators,
    },
}

#[derive(Debug, Clone)]
pub struct Catch {
    ident: String,
    kinds: Vec<Vec<String>>,
    body: Vec<Nodes>,
}