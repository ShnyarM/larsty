use crate::{Statement, Token, Expression, Node};
use super::types::{Grammar, Symbol, ProcessedSymbol, Rule, Associativity};

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
    Start 
}

impl NonTerminal {
    // UPDATE THIS!!!
    pub const COUNT: usize = 3;

    pub fn index(&self) -> usize {
        *self as usize
    }
}

// Add a variant for each type the parser creates
#[derive(Debug)]
pub enum AstNode {
    S(Node<Statement>),
    E(Node<Expression>)
}

// Define some helper function to make generators simpler
impl AstNode {
    pub fn to_expr(self) -> Node<Expression> {
        match self {
            AstNode::E(e) => e,
            _ => panic!("AstNode is not an expression")
        }
    }

    pub fn to_stmt(self) -> Node<Statement> {
        match self {
            AstNode::S(s) => s,
            _ => panic!("AstNode is not an statement")
        }
    }
}

// Helper function to make adding rules easier
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

// Make changes to grammar here
pub fn get_rules() -> Grammar {
    use Expression::*;
    use Symbol::*;
    use NonTerminal::*;
    use Token::*;
    use Associativity::*;
    
    // IMPORTANT!!
    // When adding a NonTerminal that requires data in a rule (e.g. Int(i32)), just add a random
    // value, but make sure that you ALWAYS USE SAME VALUE
    //
    // Can otherwise falsely lead to conflicts in parser

    // Define associativity of different operators
    let associativity = vec![
        (Plus, Left),
        (Minus, Left),
        (Mult, Left)
    ];

    // Define precedence of different operators. Operators with a hihger value
    // have higher precedence
    let precedence = vec![
        (Mult, 100),
        (Minus, 90),
        (Plus, 90)
    ];

    // First rule has to be a start rule that consists of a NonTerminal followed by EOF
    // Structure of each rule: rule(
    //     NonTerminal that is on LHS of rule,
    //     RHS of rule, represented as vector of Symbols,
    //     Generator that takes in a list of ProcessedSymbols that correspond
    //     to RHS and produces an AstNode
    // )
    let rules = vec![
        rule(Start, vec![N(Expr), T(EOF)],
            |mut s| s.pop().unwrap().astnode()
        ),
        rule(
            Expr,
            vec![],
            |_| AstNode::E(Node::no_loc(Val(6)))
        ),
        rule(
            Expr,
            vec![N(Expr), T(Plus), N(Expr)],
            |mut s| {
                let loc = s[0].loc().clone();
                let e2 = s.pop().unwrap().astnode().to_expr();
                s.pop(); // "+"
                let e1 = s.pop().unwrap().astnode().to_expr();
                AstNode::E(Node::new(Add(Box::new(e1), Box::new(e2)), loc))
            }
        ),
        rule(
            Expr,
            vec![N(Expr), T(Minus), N(Expr)],
            |mut s| {
                let loc = s[0].loc().clone();
                let e2 = s.pop().unwrap().astnode().to_expr();
                s.pop(); // "+"
                let e1 = s.pop().unwrap().astnode().to_expr();
                AstNode::E(Node::new(Sub(Box::new(e1), Box::new(e2)), loc))
            }
        ),
        rule(
            Expr,
            vec![T(Minus), N(Expr)],
            |mut s| {
                let loc = s[0].loc().clone();
                let e = s.pop().unwrap().astnode().to_expr();
                AstNode::E(Node::new(Neg(Box::new(e)), loc))
            }
        ),
        rule(
            Expr,
            vec![N(Expr), T(Mult), N(Expr)],
            |mut s| {
                let loc = s[0].loc().clone();
                let e2 = s.pop().unwrap().astnode().to_expr();
                s.pop(); // "+"
                let e1 = s.pop().unwrap().astnode().to_expr();
                AstNode::E(Node::new(Mul(Box::new(e1), Box::new(e2)), loc))
            }
        ),
        rule(
            Expr,
            vec![T(Int(0))],
            |mut l| {
                let v = l[0].int(); 
                let loc = l.pop().unwrap().into_loc();
                AstNode::E(Node::new(Val(v), loc))
            }
        )
    ];

    Grammar { rules, associativity, precedence }
}
