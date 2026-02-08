use std::{collections::HashMap, fmt};

use crate::{Node, Token, Location};
use super::rules::{AstNode, NonTerminal};

/// Used to describe rules in the grammar
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug)]
pub enum Symbol {
    T(Token),
    N(NonTerminal)
}

/// A symbol that has been consumed by parser. A terminal still just consists of a token, but
/// NonTerminal additionally has the AstNode generated associated to it
#[derive(Debug)]
pub enum ProcessedSymbol {
    T(Node<Token>),
    N(NonTerminal, AstNode)
}

pub type RuleGenerator = Box<dyn Fn(Vec<ProcessedSymbol>) -> AstNode>;

pub struct Rule {
    pub lhs: NonTerminal,
    pub rhs: Vec<Symbol>,
    pub generator: RuleGenerator 
}

#[derive(Clone, Copy)]
pub enum Associativity {
    Right,
    Left,
    NonAssoc
}

pub struct Grammar {
    pub rules: Vec<Rule>,
    pub precedence: Vec<(Token, usize)>,
    pub associativity: Vec<(Token, Associativity)>
} 

impl Grammar {
    pub fn get_precedence_index(&self, token_index: usize) -> usize {
        self.precedence
            .iter()
            .find(|(t, _)| t.index() == token_index)
            .map(|(_, v)| *v)
            .unwrap_or(0)
    }

    pub fn get_assoc_index(&self, token_index: usize) -> Associativity {
        self.associativity
            .iter()
            .find(|(t, _)| t.index() == token_index)
            .map(|(_, v)| *v)
            .unwrap_or(Associativity::NonAssoc)
    }

    pub fn get_rule_precedence(&self, index: usize) -> (usize, Associativity) {
        let mut last_token: &Token = &Token::EOF;
        let mut found = false;

        for s in self.rules[index].rhs.iter() {
            if let Symbol::T(t) = s {
                last_token = t;
                found = true;
            }
        }

        if !found {
            panic!("No terminal present in rule");
        }
        
        let i = last_token.index();

        (self.get_precedence_index(i), self.get_assoc_index(i))
    }
}

impl ProcessedSymbol {
    pub fn loc(&self) -> &Location {
        match self {
            ProcessedSymbol::N(_, n) => {
                match n {
                    AstNode::S(s) => s.loc(),
                    AstNode::E(e) => e.loc()
                }
            },
            ProcessedSymbol::T(t) => t.loc()
        }
    }

    pub fn into_loc(self) -> Location {
        match self {
            ProcessedSymbol::N(_, n) => {
                match n {
                    AstNode::S(s) => s.into_loc(),
                    AstNode::E(e) => e.into_loc()
                }
            },
            ProcessedSymbol::T(t) => t.into_loc()
        }
    }

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

    pub fn int(&self) -> i32 {
        match self {
            ProcessedSymbol::T(n) => {
                if let Token::Int(v) = n.get() {
                    *v
                } else {
                    panic!("Not a number")
                }
            }
            _ => panic!("Not a terminal")
        }
    }
}

impl PartialEq for Rule {
    fn eq(&self, other: &Self) -> bool {
        self.lhs == other.lhs && self.rhs == other.rhs
    }
}

impl Eq for Rule { }

impl Ord for Rule {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.lhs
            .cmp(&other.lhs)
            .then_with(|| self.rhs.cmp(&other.rhs))
    }
}

impl PartialOrd for Rule {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Debug for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Rule")
            .field("lhs", &self.lhs)
            .field("rhs", &self.rhs)
            .finish()
    }
}
