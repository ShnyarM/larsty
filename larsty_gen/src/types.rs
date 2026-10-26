// mod rules;
// mod types;
// mod automaton;
// mod parser;

// pub use parser::Parser;
// pub use rules::{get_rules, AstNode};

// pub fn pre_compile_parser() {
//     Parser::pre_compile(rules::get_rules());
// }

use larsty_rt::Associativity;

pub struct ParserRule {
    pub lhs: String,
    pub rhs: Vec<String>,
}

// Sorting implementation, needed to sort certain types
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

/// A full, final parser grammar
pub struct Grammar {
    pub rules: Vec<Rule>,
    // TO DO: Optimize this
    pub precedence: Vec<(String, usize)>, // Maps operators to precedence value
    pub associativity: Vec<(String, Associativity)> // Maps operators to associativity
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

    pub fn get_rule_precedence(&self, index: usize) -> Option<(usize, Associativity)> {
        let mut last_token: &Token = &Token::EOF;
        let mut found = false;

        for s in self.rules[index].rhs.iter() {
            if let Symbol::T(t) = s {
                last_token = t;
                found = true;
            }
        }

        if !found {
            return None;
        }
        
        let i = last_token.index();

        Some((self.get_precedence_index(i), self.get_assoc_index(i)))
    }
}
