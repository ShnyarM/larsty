mod automaton;
mod regexp;
use automaton::{NFA, DFA, EPSILON_CHAR};
use std::{fs::File, io::Write, env, path::PathBuf};

pub use regexp::Regexp;

pub struct Lexer<T> {
    dfa: DFA,
    token_generators: Vec<Box<dyn Fn(&str) -> T>>
}

impl<T> Lexer<T>{
    /// Generates the lexer for this set of rules and saves it in binary
    /// Runs when building project
    pub fn pre_compile(rules: Vec<(Regexp, Box<dyn Fn(&str) -> T>)>) {
        let lexer = Self::generate(rules);
        let compressed_dfa = lexer.dfa.compress();

        // save into file 
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let path = out_dir.join("lexer.dfa");

        let mut file = File::create(&path).unwrap();
        let json = serde_json::to_string(&compressed_dfa).unwrap();
        write!(file, "{}", json).unwrap();
    }

    pub fn generate(rules: Vec<(Regexp, Box<dyn Fn(&str) -> T>)>) -> Lexer<T> {
        let n = rules.len();
        if n < 1 {
            return Lexer { dfa: Regexp::Empty.to_dfa(), token_generators: Vec::new() }
        }

        // Create big NFA which includes all rules
        let mut nfa = NFA::new();
        let mut generators = Vec::new();
        let start = nfa.get_start_state();

        // Add whitespace transition so rules can have whitespaces between them
        nfa.add_transition(start, start, b' ');

        for (i, (regexp, token)) in rules.into_iter().enumerate() {
            // First rules should have highest action number
            // Lowest priority will have action_level==0
            let action_level = n-i-1;
            let new_nfa = regexp.to_nfa(action_level as i32);

            let merge_list = vec![
                ((0, start), (1, new_nfa.get_start_state()), EPSILON_CHAR)
            ];
            nfa.merge(&new_nfa, &merge_list);

            generators.push(token);
        }

        // Reverse actions, since first rule has highest index
        generators.reverse();
        Lexer { dfa: nfa.to_dfa(), token_generators: generators }
    }

    pub fn lex(&self, input:&str) -> Result<Vec<T>, &str> {
        let actions = self.dfa.longest_matching(input)?;

        println!("{:?}", actions);

        let result = actions.iter().map(|(action, str)| self.token_generators[*action as usize](str)).collect();
        Ok(result)
    }
}
