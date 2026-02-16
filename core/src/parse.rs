use frontend::parse::{get_rules, Parser, AstNode};
use frontend::{Token, Node, Program}; 

// Gets created during build time by frontend_generator in frontend_generator/src/parse/parse.rs
static DFA_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/parser.dfa"));

pub fn parse_input(input: Vec<Node<Token>>) -> Result<Program, String> {
    let parser = get_parser();
    let result = parser.parse(input);
    match result {
        Ok(AstNode::P(p)) => Ok(p),
        Err(e) => Err(e),
        _ => Err(String::from("Parse result is not a program. This shouldn't even happen"))
    }
}

fn get_parser() -> Parser {
   Parser::from_saved(str::from_utf8(DFA_BYTES).unwrap(), get_rules()) 
}
