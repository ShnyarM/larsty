use std::{fs::File, io::Write, env, path::PathBuf};

use super::automaton::{ParserDFA, CompressedParserDFA};
use super::types::{Grammar};
use super::rules::{AstNode};
use crate::ast::Node;
use crate::Token;

/// Represents a complete parser with the DFA and generators
pub struct Parser {
    dfa: ParserDFA,
    grammar: Grammar
}

impl Parser {
    /// Generates the parser for this set of rules and saves it in binary
    /// Runs when building project
    pub fn pre_compile(grammar: Grammar) {
        let parser = Self::generate(grammar);
        let compressed_dfa = parser.dfa.compress();

        // save into file 
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let path = out_dir.join("parser.dfa");

        let mut file = File::create(&path).unwrap();
        let json = serde_json::to_string(&compressed_dfa).unwrap();
        write!(file, "{}", json).unwrap();
    }
    
    /// Generates a complete Parser from given grammar
    pub fn generate(grammar: Grammar) -> Parser {
        let dfa = ParserDFA::generate(&grammar);

        Parser { dfa, grammar }
    }

    /// Given a json representation of a compressed DFA and the set of rules, constructs the parser
    pub fn from_saved(
        str: &str, 
        grammar: Grammar)
    -> Parser {
        // DFA is already consructed, so just read it and add generators to vec
        let compressed_dfa: CompressedParserDFA = serde_json::from_str(str).unwrap(); 
        let dfa = ParserDFA::from_compressed(compressed_dfa);
        
        return Parser { dfa, grammar }
    }

    pub fn parse(&self, input: Vec<Node<Token>>) -> Result<AstNode, String> {
        self.dfa.parse(input, &self.grammar)
    }
}
