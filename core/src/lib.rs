use frontend::{Token, Node};

mod lex;
mod parse;

pub fn compile(program: &str) {
    let lexer = lex::get_lexer();
    println!("{}", program);

    println!("Starting lex");

    let mut tokens = lexer.lex(program).unwrap();
    tokens.push(Node::no_loc(Token::EOF));
    println!("{:?}", tokens);

    println!("Completed lex");

    let parser = parse::get_parser();
    let ast = parser.parse(tokens).unwrap();
    println!("{:?}", ast);

    println!("Result: {}", ast.to_expr().get().eval());
}
