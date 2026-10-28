use std::collections::HashMap;
use larsty_rt::Associativity;

use crate::Regexp;
use crate::{ParserRule, Grammar, LexerRule};
use crate::lex::{NFA, GenDFA};

/// Used to build a grammar
///
/// User creates an object of this type, and inserts its rules into this
/// object.
///
/// Generating a spec will create both the lexer and parser for the runtime
/// to use.
struct Spec<'a> {
    name: &'a str,

    /// Lexer tokens. Associate each one with an id.
    ///
    /// The lower the id, the higher the precedence of the token.
    token_count: u32,
    tokens: HashMap<String, u32>, 

    /// Parser nonterminals. Associate each one with an id
    non_terminal_count: u32,
    non_terminals: HashMap<String, u32>,

    lexer_rules: Vec<Regexp>,
    parser_rules: Grammar,

    current_precedence_level: usize
}

impl Spec {

    // ---- PUBLIC ----

    pub fn spec(name: &str) -> Spec{
        Spec {
            name: name,

            token_count: 0,
            tokens: HashMap::new(),

            non_terminal_count: 0,
            non_terminals: HashMap::new(),

            // Index of rule in this vec determines for which token
            // this rule applies
            lexer_rules: Vec::new(), 
            parser_rules: Grammar {
                rules: Vec::new(),
                precedence: Vec::new(),
                associativity: Vec::new()
            },

            current_precedence_level: usize::MAX
        }
    }

    /// Add a new lexer token.
    ///
    /// Takes the name of the token, together with the regular expression
    /// that describes the token.
    pub fn token(&mut self, name: &str, rule: Regexp){
        self.tokens.insert(String::from(name), self.token_count);
        self.token_count += 1;
        self.lexer_rules.push(rule);
    }

    /// Helper function that generates a lexer token whose regular expression
    /// is a simple string.
    ///
    /// Equivalent to calling token(name, Regexp::String(keyword))
    pub fn keyword(&mut self, name: &str, keyword: &str){
        self.token(name, Regexp::String(String::from(keyword)));
    }
    
    /// Makes a set of tokens left associative and sets their precedence
    /// to the same level.
    ///
    /// Mainly used to resolve operator grammar conflicts.
    pub fn left(&mut self, tokens: &Vec<&str>){
        self.add_associativity(tokens, Associativity::Right);
    }

    /// Makes a set of tokens right associative and sets their precedence
    /// to the same level.
    ///
    /// Mainly used to resolve operator grammar conflicts.
    pub fn right(&mut self, tokens: &Vec<&str>){
        self.add_associativity(tokens, Associativity::Left);
    }

    /// Adds a grammar rule to the spec.
    ///
    /// Implicitly adds the left-hand side to the list of nonterminals.
    pub fn rule(&mut self, lhs: &str, rhs: Vec<&str>){
        self.add_nonterminal(lhs);
        
        // TO DO: maybe optimize this with more performant ownership rules
        let mut rhs_vec = Vec::new();
        
        for &rhs_rule in rhs {
            rhs_vec.push(String::from(rhs_rule));
        }

        self.parser_rules.rules.push(ParserRule {
            lhs: String::from(lhs),
            rhs: rhs_vec
        })
    }

    pub fn generate(&self){
        
    }

    // ---- PRIVATE ----

    fn add_associativity(&mut self, tokens: &Vec<&str>, assoc: Associativity){
        for &tok in tokens {
            if(!self.tokens.contains_key(tok)){
                panic!("Adding associativity rules requires the token to be defined. Token {} has not been defined yet!", tok);
            }
            self.parser_rules.associativity.push((String::from(tok), assoc));
            self.parser_rules.precedence.push((String::from(tok), self.current_precedence_level));
        } 
        self.current_precedence_level -= 1;
    }

    fn add_nonterminal(&mut self, nonterminal: &str){
        if(self.non_terminals.contains_key(nonterminal)) {
            return;
        }
        
        self.non_terminals.insert(String::from(nonterminal), self.non_terminal_count);
        self.non_terminal_count += 1;
    }

    fn generate_lexer(&self){
        let rules = self.lexer_rules();
        let n = rules.len();

        // TO DO: handle this
        // if n < 1 {
        //     return Lexer { dfa: Regexp::Empty.to_dfa(), token_generators: Vec::new() }
        // }

        // Create big NFA which includes all rules
        let mut nfa = NFA::new();
        let start = nfa.get_start_state();

        // Add whitespace transition so rules can have whitespaces between them
        nfa.add_transition(start, start, b' ');

        for (i, regexp) in rules.into_iter().enumerate() {
            // First rules should have highest action number
            // Lowest priority will have action_level==0
            let action_level = n-i-1;
            let new_nfa = regexp.to_nfa(action_level as i32);

            let merge_list = vec![
                ((0, start), (1, new_nfa.get_start_state()), EPSILON_CHAR)
            ];
            nfa.merge(&new_nfa, &merge_list);

            // generators.push(token);
        }
        let final_dfa = nfa.to_dfa();

        // Reverse actions, since first rule has highest index
        // generators.reverse();
        println!("{}", 5);
        // TO DO: Output the DFA
        
    }
}
