mod automaton;
mod regexp;
use automaton::{NFA, DFA, EPSILON_CHAR, NOT_ACCEPT};

pub use regexp::Regexp;

pub struct Lexer<T> {
    dfa: DFA,
    token_generators: Vec<Box<dyn Fn(&str) -> T>>
}

impl<T> Lexer<T>{
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

impl Regexp {
    pub fn to_dfa(&self) -> DFA {
        self.to_nfa(0).to_dfa()
    }

    /// Create NFA. action is value of action associated with accept states of this NFA.
    fn to_nfa(&self, action: i32) -> NFA {
        let mut nfa = NFA::new();

        match self {
            Regexp::Empty => {
                // Set starting state as accept state
                nfa.set_action(nfa.get_start_state(), action);       
            },
            Regexp::String(s) => {
                let length = s.len();
                nfa.set_state_count(length+1);

                // add transitions
                for (i, b) in s.bytes().enumerate() {
                   nfa.add_transition(i, i+1, b); 
                }

                // Set accept state
                nfa.set_action(length, action);
            },
            Regexp::Or(r1, r2) => {
                let nfa1 = r1.to_nfa(action);
                let nfa2 = r2.to_nfa(action);

                let start = nfa.get_start_state();
                let nfa1_start = nfa1.get_start_state();
                let nfa2_start = nfa2.get_start_state();

                // Define transitions for merges
                let merge1_list = vec![
                    ( (0, start), (1, nfa1_start), EPSILON_CHAR )
                ];
                let merge2_list = vec![
                    ( (0, start), (1, nfa2_start), EPSILON_CHAR )
                ];

                nfa.merge(&nfa1, &merge1_list).merge(&nfa2, &merge2_list);
            },
            Regexp::Concat(r1, r2) => {
                let mut nfa1 = r1.to_nfa(action);
                let nfa2 = r2.to_nfa(action);
                let nfa2_start = nfa2.get_start_state();
                
                let mut merge_list = vec![];
                for s in nfa1.get_accept_states() {
                    merge_list.push(
                        ((0, s), (1, nfa2_start), EPSILON_CHAR)
                    );

                    // Remove accept
                    nfa1.set_action(s, NOT_ACCEPT);
                }

                nfa1.merge(&nfa2, &merge_list);
                
                nfa=nfa1;
            },
            Regexp::Rep(r) => {
                let nfa1 = r.to_nfa(action);
                let nfa1_start = nfa1.get_start_state();
                let start = nfa.get_start_state();

                let mut merge_list = vec![];
                for s in nfa1.get_accept_states() {
                    merge_list.push(
                        ((0, start), (1, s), EPSILON_CHAR)
                    );
                    merge_list.push(
                        ((1, s), (1, nfa1_start), EPSILON_CHAR)
                    );
                }

                nfa.merge(&nfa1, &merge_list);
            }
        };

        nfa
    }
}


#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn lex_hello_world() {
        let regexp = Regexp::string("Hello World!"); 
        let dfa = regexp.to_dfa();

        assert!(dfa.execute("Hello World!"));
        assert!(!dfa.execute("faasdfHello World!"));
        assert!(!dfa.execute("Hello World!a"));
        assert!(!dfa.execute(""));
    }

    #[test]
    fn lex_empty() {
        let dfa = Regexp::string("").to_dfa();

        assert!(dfa.execute(""));
        assert!(!dfa.execute("Hello"));

        let dfa = Regexp::empty().to_dfa();
        assert!(dfa.execute(""));
        assert!(!dfa.execute("Hello"));
    }

    #[test]
    fn lex_or() {
        let dfa = Regexp::or(
            Regexp::string("This is one."),
            Regexp::string("Another one..")
        ).to_dfa();

        assert!(dfa.execute("This is one."));
        assert!(dfa.execute("Another one.."));
        assert!(!dfa.execute("Hello"));
        assert!(!dfa.execute("This is different."));
        assert!(!dfa.execute(""));
    }

    #[test]
    fn lex_concat() {
        let dfa = Regexp::concat(
            Regexp::string("This is one."),
            Regexp::string("Another one..")
        ).to_dfa();

        assert!(dfa.execute("This is one.Another one.."));
        assert!(!dfa.execute("This is one."));
        assert!(!dfa.execute("Another one.."));
        assert!(!dfa.execute("Hello"));
        assert!(!dfa.execute("This is different."));
        assert!(!dfa.execute(""));
    }

    #[test]
    fn lex_rep() {
        let dfa = Regexp::rep(
            Regexp::string("repetition")
        ).to_dfa();

        assert!(dfa.execute(""));
        assert!(dfa.execute("repetition"));
        assert!(dfa.execute("repetitionrepetition"));
        assert!(dfa.execute("repetitionrepetitionrepetition"));
        assert!(!dfa.execute("repetitionrepetitionrepe"));
        assert!(!dfa.execute("repetonrepetition"));
        assert!(!dfa.execute("Hello"));
        assert!(!dfa.execute("This is different."));
    }

    #[test]
    fn lex_complex_expression() {
        // ((ab | cd) e*) f
        let regex = Regexp::concat(
            Regexp::concat(
                Regexp::or(
                    Regexp::string("ab"),
                    Regexp::string("cd"),
                ),
                Regexp::rep(
                    Regexp::string("e"),
                ),
            ),
            Regexp::string("f"),
        );

        let dfa = regex.to_dfa();

        // accepted
        assert!(dfa.execute("abf"));
        assert!(dfa.execute("abef"));
        assert!(dfa.execute("abeeef"));
        assert!(dfa.execute("cdf"));
        assert!(dfa.execute("cdef"));
        assert!(dfa.execute("cdeeeeeef"));

        // rejected
        assert!(!dfa.execute("ab"));
        assert!(!dfa.execute("cd"));
        assert!(!dfa.execute("abefg"));
        assert!(!dfa.execute("ef"));
        assert!(!dfa.execute("abeee"));
        assert!(!dfa.execute("hello"));
    }
}
