use std::vec;

use crate::Statement;
use crate::Token;
use crate::Expression;
use crate::Node;

use super::types::Symbol;
use super::types::ProcessedSymbol;
use super::types::Rule;

// HOW TO ADD RULES 
// Rules have to be added in the get_rules() function.
// Each possible NonTerminal has to be added to the NonTerminal enum. Make sure to adjust
// NON_TERMINAL_COUNT accordingly.
// 
// Each rule needs a generator that specifies how to convert the Non_Terminal to an element that
// is part of Ast. Output has to be an AstNode, so if generator returns a type that can't be
// represented by an AstNode yet, then add that type to AstNode enum.

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Debug)]
#[repr(u8)]
pub enum NonTerminal {
    Expr,
    Stmt,
    SPrime, S
}
pub const NON_TERMINAL_COUNT: usize = 4;

impl NonTerminal {
    pub fn index(&self) -> usize {
        *self as usize
    }
}

#[derive(Debug)]
pub enum AstNode {
    S(Statement),
    E(Expression)
}

fn rule<F>(
    lhs: NonTerminal,
    rhs: Vec<Symbol>,
    reduce: F
) -> Rule
where
    F: Fn(Vec<ProcessedSymbol>) -> AstNode + 'static
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
    /*vec!{
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
    }*/
    vec![
        rule(
            NonTerminal::Expr,
            vec![N(NonTerminal::Expr), T(Token::EOF)],
            |mut s| {
                let e = s.pop().unwrap().convert();
                let (n, _) = e.extract();
                n
            }
        ),
        rule(
            NonTerminal::Expr,
            vec![N(NonTerminal::Expr), T(Token::Plus), T(Token::Int(0))],
            |mut s| {
                let e2 = s.pop().unwrap();
                let e2_loc = e2.get_loc().clone();
                let e2 = Box::new(Node::new(Val(e2.int()), e2_loc));
                s.pop(); // "+"
                let e1 = s.pop().unwrap().convert();
                let e1 = Box::new(ast_node_to_expr(e1));
                AstNode::E(Add(e1, e2))
            }
        ),
        rule(
            NonTerminal::Expr,
            vec![T(Token::Int(0))],
            |mut l| {
                let v = l.pop().unwrap().int(); 
                AstNode::E(Val(v))
            }
        )
    ]
}
