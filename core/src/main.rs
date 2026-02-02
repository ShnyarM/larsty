fn main() {
    let lexer = core::lex::get_lexer();

    let tokens = lexer.lex("if function ifififif hellou").unwrap();
    println!("{:?}", tokens);
}

