mod automaton;

pub use lexer::Lexer;
pub use rules::get_rules;
pub use automaton::{DFA, CompressedDFA};

pub fn pre_compile_lexer() {
    Lexer::pre_compile(rules::get_rules());
}
