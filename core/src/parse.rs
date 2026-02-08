use frontend::parse::{get_rules, Parser};

// Gets created during build time by frontend_generator in frontend_generator/src/parse/parse.rs
static DFA_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/parser.dfa"));

pub fn get_parser() -> Parser {
   Parser::from_saved(str::from_utf8(DFA_BYTES).unwrap(), get_rules()) 
}
