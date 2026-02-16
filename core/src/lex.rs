use frontend::lex::{get_rules, Lexer};
use frontend::{Token, Node};

// Gets created during build time by frontend_generator in frontend_generator/src/lex/lexer.rs
static DFA_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/lexer.dfa"));

pub fn lex_input(input: &str) -> Result<Vec<Node<Token>>, String> {
    let lexer = get_lexer();
    let result = lexer.lex(input);

    match result {
        Ok(mut tokens) => {
            tokens.push(Node::no_loc(Token::EOF));
            Ok(tokens)
        },
        Err(e) => Err(e) 
    }
}

fn get_lexer() -> Lexer<Token> {
   Lexer::from_saved(str::from_utf8(DFA_BYTES).unwrap(), get_rules()) 
}
