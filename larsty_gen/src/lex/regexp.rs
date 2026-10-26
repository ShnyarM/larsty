pub enum Regexp{
    Empty,
    String(String),
    Or(Box<Regexp>, Box<Regexp>),
    Concat(Box<Regexp>, Box<Regexp>),
    Rep(Box<Regexp>)
}

impl Regexp {
    pub fn empty() -> Regexp {
        Regexp::Empty
    }

    pub fn string(str: &str) -> Regexp {
        Regexp::String(String::from(str))
    }

    pub fn or(r1: Regexp, r2: Regexp) -> Regexp {
        Regexp::Or(Box::new(r1), Box::new(r2))
    }

    pub fn concat(r1: Regexp, r2: Regexp) -> Regexp {
        Regexp::Concat(Box::new(r1), Box::new(r2))
    }

    pub fn rep(r: Regexp) -> Regexp {
        Regexp::Rep(Box::new(r))
    }

    pub fn to_dfa(&self) -> DFA {
        self.to_nfa(0).to_dfa()
    }

    /// Creates an big Or of a list of chars
    pub fn char_class(chars: impl IntoIterator<Item = char>) -> Regexp {
        chars
            .into_iter()
            .map(|c| Self::String(c.to_string()))
            .reduce(Self::or)
            .unwrap()
    }

    pub fn lowercase() -> Regexp {
        Self::char_class('a'..='z')
    }
    
    pub fn uppercase() -> Regexp {
        Self::char_class('A'..='Z')
    }

    pub fn letter() -> Regexp {
        Self::or(Self::lowercase(), Self::uppercase())
    }

    pub fn digit() -> Regexp {
        Self::char_class('0'..='9')
    }

    /// Create NFA. action is value of action associated with accept states of this NFA.
    pub fn to_nfa(&self, action: i32) -> NFA {
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
