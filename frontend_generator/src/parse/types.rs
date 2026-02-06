use std::fmt;

use crate::{Node, Token, Location};
use super::rules::{AstNode, NonTerminal};

/// Used to describe rules in the grammar
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug)]
pub enum Symbol {
    T(Token),
    N(NonTerminal)
}

/// A symbol that has been consumed by parser. A terminal still just consists of a token, but
/// NonTerminal additionally has the ProcessedSymbols that will be fed to generator to turn it into
/// a AstNod, as well as index of generator that has to be used on it.
/// Will be the output of the ParserDFA
pub enum ProcessedSymbol {
    T(Node<Token>),
    N(Node<(NonTerminal, AstNode)>)
}

pub type RuleGenerator = Box<dyn Fn(Vec<ProcessedSymbol>) -> AstNode>;

pub struct Rule {
    pub lhs: NonTerminal,
    pub rhs: Vec<Symbol>,
    pub generator: RuleGenerator 
}


pub type Grammar = Vec<Rule>;

impl ProcessedSymbol {
    pub fn get_loc(&self) -> &Location {
        match self {
            ProcessedSymbol::T(i) => i.loc(),
            ProcessedSymbol::N(i) => i.loc()
        }
    }

    pub fn convert(self) -> Node<AstNode>{
        match self {
            ProcessedSymbol::N(n) => {
                let ((_, i), loc) = n.extract();
                Node::new(i, loc)
            }
            _ => panic!("Can't convert a terminal.")
        }
    }

    pub fn int(self) -> i32 {
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
