use super::types::{Grammar, Symbol, ProcessedSymbol, Rule, Associativity};
use crate::Token;
use crate::ast;
use crate::ast::*;

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
    Type,
    RetType,
    Expr,
    Stmt,
    StmtList,
    Function,
    Program,
    Start 
}

impl NonTerminal {
    // UPDATE THIS!!!
    pub const COUNT: usize = 7;

    pub fn index(&self) -> usize {
        *self as usize
    }
}

// Add a variant for each type the parser creates
#[derive(Debug)]
pub enum AstNode {
    T(Node<Type>),
    RT(Node<RetType>),
    S(Node<Statement>),
    SL(StatementList),
    E(Node<Expression>),
    F(Node<FunctionDecl>),
    P(Program)
}

// Define some helper function to make generators simpler
impl AstNode {
    pub fn expr(self) -> Node<Expression> {
        match self {
            AstNode::E(e) => e,
            _ => panic!("AstNode is not an expression")
        }
    }

    pub fn stmt(self) -> Node<Statement> {
        match self {
            AstNode::S(s) => s,
            _ => panic!("AstNode is not a statement")
        }
    }

    pub fn stmt_list(self) -> StatementList {
        match self {
            AstNode::SL(s) => s,
            _ => panic!("AstNode is not a statement list")
        }
    }

    pub fn function(self) -> Node<FunctionDecl> {
        match self {
            AstNode::F(s) => s,
            _ => panic!("AstNode is not a function")
        }
    }

    pub fn program(self) -> Program {
        match self {
            AstNode::P(s) => s,
            x => panic!("AstNode is not a program, it is: {:?}", x)
        }
    }

    pub fn get_type(self) -> Node<Type> {
        match self {
            AstNode::T(s) => s,
            _ => panic!("AstNode is not a type")
        }
    }

    pub fn ret_type(self) -> Node<RetType> {
        match self {
            AstNode::RT(s) => s,
            _ => panic!("AstNode is not a return type")
        }
    }
}

// Some helper functions for generators
impl ProcessedSymbol {
    pub fn astnode(self) -> AstNode{
        match self {
            ProcessedSymbol::N(_, n) => n, 
            _ => panic!("ProcessedSymbol conversion to astnode tried on token")
        }
    }

    pub fn token(self) -> Node<Token> {
        match self {
            ProcessedSymbol::T(t) => t,
            _ => panic!("ProcessedSymbol convertsion to token tried on NonTerminal")
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
        rule(Start,
            vec![N(Program), T(EOF)],
            |mut s| s.pop().unwrap().astnode()
        ),

        // ------------- PROGRAM ----------------
        rule(
            Program,
            vec![N(Program), N(NonTerminal::Function)],
            |mut l| {
                let f = l.pop().unwrap().astnode().function();
                let mut p = l.pop().unwrap().astnode().program();
                p.add_function(f);
                AstNode::P(p)
            }
        ),
        rule(
            Program,
            vec![],
            |_| AstNode::P(ast::Program::new())
        ),

        // ------------- FUNCTION ----------------
        rule(
            NonTerminal::Function,
            vec![N(RetType), T(Token::Id(String::from(""))), T(LParen), T(RParen), T(LBrack), N(StmtList), T(RBrack)],
            |mut l| {
                l.pop(); // }
                let list = l.pop().unwrap().astnode().stmt_list(); // statements
                l.pop(); // {
                l.pop(); // )
                l.pop(); // (
                let name = l.pop().unwrap().token().into_get().str(); // function name
                let (ret_type, loc) = l.pop().unwrap().astnode().ret_type().extract();
                AstNode::F(
                    Node::new(
                        FunctionDecl::new(name, ret_type, list),
                        loc
                    )
                )
            }
        ),

        // ------------- STATEMENT LIST ----------------
        rule(
            NonTerminal::StmtList,
            vec![N(NonTerminal::StmtList), N(NonTerminal::Stmt)],
            |mut l| {
                let stmt = l.pop().unwrap().astnode().stmt();
                let mut list = l.pop().unwrap().astnode().stmt_list();
                list.push(stmt);
                AstNode::SL(list)
            }
        ),

        rule(
            NonTerminal::StmtList,
            vec![],
            |_| AstNode::SL(Vec::new())
        ),

        // ------------- STATEMENT ----------------
        rule(   // --- DECLARATION ---
            NonTerminal::Stmt,
            vec![T(Token::Let), T(Token::Id(String::from(""))), T(Token::Eq), N(NonTerminal::Expr), T(Token::SemiColon)],
            |mut l| {
                l.pop(); // ;
                let expr = l.pop().unwrap().astnode().expr();
                l.pop(); // ;
                let name = l.pop().unwrap().token().into_get().str();
                let loc = l.pop().unwrap().token().into_loc();
                AstNode::S(Node::new(Statement::Decl(name, expr), loc))
            }
        ),

        // ------------- EXPRESSION ----------------
        rule(
            NonTerminal::Expr,
            vec![T(Token::Int(0))],
            |mut l| {
                let (v, loc) = l.pop().unwrap().token().extract();
                AstNode::E(Node::new(Expression::Val(v.i32()), loc))
            }
        ),

        // ------------- TYPE ----------------
        rule(
            NonTerminal::Type,
            vec![T(Token::I32)],
            |mut l| {
                let loc = l.pop().unwrap().token().into_loc();
                AstNode::T(Node::new(ast::Type::I32, loc))
            } 
        ),

        // ------------- RETURN TYPE ----------------
        rule(
            NonTerminal::RetType,
            vec![N(NonTerminal::Type)],
            |mut l| {
                let (t, loc) = l.pop().unwrap().astnode().get_type().extract();
                AstNode::RT(Node::new(ast::RetType::Type(t), loc))
            }
        ),
        rule(
            NonTerminal::RetType,
            vec![T(Token::Void)],
            |mut l| {
                let loc = l.pop().unwrap().token().into_loc();
                AstNode::RT(Node::new(ast::RetType::Void, loc))
            }
        )
    ];

    Grammar { rules, associativity, precedence }
}
