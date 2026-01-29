use std::vec;

//use rust_parser::types::Expression;
use rust_parser::lex::{Lexer, Regexp};

#[derive(Debug)]
enum Token {
    If,
    Function,
    Id(String)
}

fn main() {
    use Token::*;

    let rules: Vec<(Regexp, Box<dyn Fn(&str) -> Token>)> = vec![
        (Regexp::string("if"), Box::new(|_| If)),
        (Regexp::string("if"), Box::new(|_| Function)),
        (Regexp::string("function"), Box::new(|_| Function)),
        (Regexp::string("identifier"), Box::new(|s| {
            let str = String::from(s).trim().to_string();
            Id(str)
        }))
    ];

    let lexer = Lexer::generate(rules);

    let list = lexer.lex("if if function if identifier function");
    
    println!("{:?}", &list);
}
