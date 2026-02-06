use std::vec;

use crate::Statement;
use crate::Token;
use crate::Expression;
use crate::Node;

use super::types::Grammar;
use super::types::AstNode;
use super::types::Symbol;
use super::types::ProcessedSymbol;
use super::types::Rule;

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Debug)]
#[repr(u8)]
pub enum NonTerminal {
    Expr,
    Stmt,
    SPrime,
    S
}
pub const NON_TERMINAL_COUNT: usize = 4;

impl NonTerminal {
    pub fn index(&self) -> usize {
        *self as usize
    }
}


fn rule<F>(
    lhs: NonTerminal,
    rhs: Vec<Symbol>,
    reduce: F
) -> Rule
where
    F: Fn(Vec<ProcessedSymbol>, &Grammar) -> AstNode + 'static
{
    Rule {
        lhs,
        rhs,
        generator: Box::new(reduce)
    }
}

fn ast_node_to_expr(n: Node<AstNode>) -> Node<Expression> {
    let (a, l) = n.extract();
    match a {
        AstNode::E(e) => Node::new(e, l),
        _ => panic!("Not an expression")
    }
}

fn ast_node_to_stmt(n: Node<AstNode>) -> Node<Statement> {
    let (a, l) = n.extract();
    match a {
        AstNode::S(s) => Node::new(s, l),
        _ => panic!("Not a statement")
    }
}

pub fn get_rules() -> Vec<Rule> {
    use Expression::*;
    use Symbol::*;
    
    // IMPORTANT!!
    // When adding a NonTermianl that requires data in a rule (e.g. Int(i32)), just add a random
    // value, but make sure that you ALWAYS USE SAME VALUE
    //
    // Can otherwise falsely lead to conflicts in parser
    vec!{
        rule(
            NonTerminal::SPrime,
            vec![N(NonTerminal::S)],
            |_, _| AstNode::E(Val(5))
        ),
        rule(
            NonTerminal::S,
            vec![T(Token::Function), N(NonTerminal::S), T(Token::Let)],
            |_, _| AstNode::E(Val(5))),
        rule(
            NonTerminal::S,
            vec![T(Token::Eq)],
            |_, _| AstNode::E(Val(5))
        ),
    }
    /*vec![
        rule(
            NonTerminal::Expr,
            vec![N(NonTerminal::Expr), T(Token::Plus), T(Token::Int(0))],
            |mut s, g| {
                let e2 = s.pop().unwrap().convert(g);
                let e2 = Box::new(ast_node_to_expr(e2));
                s.pop(); // "+"
                let e1 = s.pop().unwrap().convert(g);
                let e1 = Box::new(ast_node_to_expr(e1));
                AstNode::E(Add(e1, e2))
            }
        ),
        rule(
            NonTerminal::Expr,
            vec![T(Token::Int(0))],
            |mut l, _| {
                let v = l.pop().unwrap().int(); 
                AstNode::E(Val(v))
            }
        )
    ]*/
}
