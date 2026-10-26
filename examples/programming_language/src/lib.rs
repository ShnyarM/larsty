mod lex;
mod parse;

pub fn compile(program: &str) -> Result<String, String> {
    let tokens = lex::lex_input(program)?;
    let program = parse::parse_input(tokens)?;
    println!("{:?}", program);

    Ok(String::from("Success"))
}
