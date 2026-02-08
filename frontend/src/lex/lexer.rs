use std::{fs::File, io::Write, env, path::PathBuf};

use super::automaton::{NFA, DFA, CompressedDFA, EPSILON_CHAR};
use super::regexp::Regexp;
use crate::ast::Node;

/// Models a complete lexer. Includes the dfa and the actions for each rule.
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
    
    /// Generates the lexer for a set of given rules
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
    
    /// Given a json representation of a compressed DFA and the set of rules, constructs the Lexer
    pub fn from_saved(
        str: &str, 
        rules: Vec<(Regexp, Box<dyn Fn(&str) -> T>)>)
    -> Lexer<T> {
        // DFA is already consructed, so just read it and add generators to vec
        let compressed_dfa: CompressedDFA = serde_json::from_str(str).unwrap(); 
        let dfa = DFA::from_compressed(compressed_dfa);
        
        let mut generators = Vec::new();
        for (_, token) in rules {
            generators.push(token);
        };
        
        // Reverse actions, since first rule has highest index
        generators.reverse();
        return Lexer { dfa, token_generators: generators }
    }

    pub fn lex(&self, input:&str) -> Result<Vec<Node<T>>, &str> {
        let actions = self.dfa.longest_matching(input)?;

        println!("{:?}", actions);

        let result = actions
            .into_iter()
            .map(|n| {
                let ((action, str), loc) = n.extract();
                let token = self.token_generators[action as usize](str);
                Node::new(token, loc)
            })
            .collect();
        
        Ok(result)
    }
}
