mod rules;
mod types;
mod automaton;
mod parser;

pub use parser::Parser;
pub use rules::get_rules;

pub fn pre_compile_parser() {
    Parser::pre_compile(rules::get_rules());
}
