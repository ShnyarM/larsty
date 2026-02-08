use frontend::lex::{get_rules, Lexer};
use frontend::Token;

// Gets created during build time by frontend_generator in frontend_generator/src/lex/lexer.rs
static DFA_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/lexer.dfa"));

pub fn get_lexer() -> Lexer<Token> {
   Lexer::from_saved(str::from_utf8(DFA_BYTES).unwrap(), get_rules()) 
}
