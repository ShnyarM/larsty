mod lex;
mod parse;

pub fn compile(program: &str) {
    let lexer = lex::get_lexer();
    println!("{}", program);

    println!("Starting lex");

    let tokens = lexer.lex(program).unwrap();
    println!("{:?}", tokens);

    println!("Completed lex");

    let _parser = parse::get_parser();
}
