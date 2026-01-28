//use rust_parser::types::Expression;
use rust_parser::lex::Regexp;

fn main() {
    let dfa = Regexp::new_rep(
        Regexp::new_string("repetition")
    ).to_dfa();

    let result = dfa.execute("Helluo");
    println!("Value: {}", result);
}
