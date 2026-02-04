mod types;
mod lex;

pub fn compile(program: &str) {
    let lexer = lex::get_lexer();
    println!("{}", program);

    let tokens = lexer.lex(program).unwrap();
    println!("{:?}", tokens);
}
