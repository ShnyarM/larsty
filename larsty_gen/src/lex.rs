mod automaton;
mod regexp;

pub use regexp::Regexp;

pub type LexerRule = Regexp;
