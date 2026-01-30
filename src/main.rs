use rust_parser::frontend;

#[derive(Debug)]
enum Token {
    If,
    Function,
    Id(String)
}

fn main() {
    frontend::pre_compile_lexer();
}
