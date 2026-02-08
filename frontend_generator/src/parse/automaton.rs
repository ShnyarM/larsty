use std::collections::{VecDeque, HashMap};

use serde::{Serialize, Deserialize};

use super::types::{Symbol, Grammar, Rule, ProcessedSymbol};
use super::rules::{NON_TERMINAL_COUNT, AstNode};
use crate::ast::Node;
use crate::token::{Token, TOKEN_COUNT};

const INVALID_SYMBOL: usize = usize::MAX;

/// A rule inside of a state. Only used during construction
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug)]
struct ParserStateRule<'a> {
    pub index: usize, // Index of rule in grammar
    pub rule: &'a Rule,
    pub lookahead: Vec<Token>,
    pub dot_pos: usize, // 0 represents right at the beginning. Tells us which symbol is in front
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
enum Action {
    None,
    Shift(usize),
    Reduce(usize), // Index of rule to be applied
}

#[derive(Debug)]
pub struct ParserDFA {
    state_count: usize,
    goto_table: Vec<[usize; NON_TERMINAL_COUNT]>,
    action_table: Vec<[Action; TOKEN_COUNT]>,
}

#[derive(Serialize, Deserialize)]
pub struct CompressedParserDFA {
    state_count: usize,
    goto_table: Vec<HashMap<usize, usize>>,
    action_table: Vec<HashMap<usize, Action>>
}

// TO DO: Make this more efficient by not always recomputing
fn first(symbol: &Symbol, grammar: &Grammar) -> Vec<Token> {
    match symbol {
        Symbol::T(t) => vec![t.clone()],
        Symbol::N(n) => {
            let mut result = Vec::new();
            for r in grammar {
                if r.lhs == *n {
                    // If we have a panic here this means we have an epsilon rule, not yet
                    // supported
                    result.append(&mut first(&r.rhs[0], grammar));
                }
            } 
            result
        }
    }
}

/// Compute the closure for a set of rules
fn state_closure<'a>(state_rules: Vec<ParserStateRule<'a>>, grammar: &'a Grammar) -> Vec<ParserStateRule<'a>> {
    let mut state_rules = state_rules;
    let mut to_process: VecDeque<usize> = VecDeque::new();
    
    for (i, _) in state_rules.iter().enumerate() {
        to_process.push_back(i);
    }
    
    // Process all rules in queue
    while let Some(v) = to_process.pop_front() {
        let r = state_rules.get(v).unwrap();
        // Check if NonTerminal in front of dot
        if let Some(Symbol::N(n)) = &r.rule.rhs.get(r.dot_pos as usize) {
            // Compute lookahead
            let mut lookahead = Vec::new();
            
            // Check if there is symbol ahead
            if let Some(s) = &r.rule.rhs.get((r.dot_pos+1) as usize) {
                lookahead.append(&mut first(s, grammar));
            } else { 
                // otherwise use already existing lookahead
                lookahead = r.lookahead.clone();
            }

            for (i, g) in grammar.iter().enumerate() {
                if g.lhs == *n { // We have found rule with same nonterminal, add
                    let new_rule = ParserStateRule::new(&grammar, i, lookahead.clone());
                    
                    // If we have rule that already has same core, just add lookahead
                    if let Some(other) = state_rules.iter_mut().find(|h| new_rule.eq_core(h)) {
                        other.lookahead.append(&mut lookahead.clone());

                        // Remove duplicates in lookahead
                        other.lookahead.sort();
                        other.lookahead.dedup();
                    } else if !state_rules.contains(&new_rule){
                        state_rules.push(new_rule);
                        to_process.push_back(state_rules.len()-1);
                    }
                }
            }     
        }
    }

    state_rules.sort();
    state_rules
}

impl<'a> ParserStateRule<'a> {
    /// Returns a ParserStateRule that is based on rule. Rule is given by grammar and index in
    /// grammar
    fn new<'t>(grammar: &'t Grammar, index: usize, lookahead: Vec<Token>) -> ParserStateRule<'t> {
        ParserStateRule {
            index,
            rule: &grammar[index],
            lookahead,
            dot_pos: 0
        }
    }

    /// Returns copy of rule, but with the dot moved by one spot
    fn increment<'t>(&self, grammar: &'t Grammar) -> ParserStateRule<'t> {
        ParserStateRule { 
            index: self.index, 
            rule: &grammar[self.index], 
            lookahead: self.lookahead.clone(), 
            dot_pos: self.dot_pos+1 
        }
    }

    /// Checks if two state rules have same core, i.e. everything same except for lookahead
    fn eq_core(&self, other: &Self) -> bool {
        self.index == other.index && self.rule == other.rule && self.dot_pos == other.dot_pos
    }
}

impl ParserDFA {
    fn new() -> ParserDFA {
        ParserDFA { 
            state_count: 0, 
            goto_table: Vec::new(), 
            action_table: Vec::new() 
        }
    }

    pub fn compress(&self) -> CompressedParserDFA {
        // Copy goto
        let mut goto_table = Vec::new();
        for s1 in self.goto_table.iter() {
            let mut map = HashMap::new();
            for (non_terminal, s2) in s1.iter().enumerate() {
                if *s2 != INVALID_SYMBOL {
                   map.insert(non_terminal, *s2); 
                }
            } 
            goto_table.push(map);
        }

        // Copy action 
        let mut action_table = Vec::new();
        for s1 in self.action_table.iter() {
            let mut map = HashMap::new();
            for (token, action) in s1.iter().enumerate() {
                if *action != Action::None {
                   map.insert(token, *action); 
                }
            } 
            action_table.push(map);
        }

        CompressedParserDFA { state_count: self.state_count, action_table, goto_table }
    }
    
    pub fn from_compressed(compressed: CompressedParserDFA) -> ParserDFA {
        // Copy goto
        let mut goto_table = Vec::new();
        for (i, t) in compressed.goto_table.iter().enumerate() {
            goto_table.push([INVALID_SYMBOL; NON_TERMINAL_COUNT]);
            for (symbol, s2) in t.iter() {
                goto_table[i][*symbol] = *s2;
            } 
        }

        // Copy actions
        let mut action_table = Vec::new();
        for (i, t) in compressed.action_table.iter().enumerate() {
            action_table.push([Action::None; TOKEN_COUNT]);
            for (symbol, s2) in t.iter() {
                action_table[i][*symbol] = *s2;
            } 
        }

        ParserDFA { state_count: compressed.state_count, goto_table, action_table }
    }
    
    fn add_state(&mut self) {
        self.state_count += 1;
        self.goto_table.push([INVALID_SYMBOL; NON_TERMINAL_COUNT]);
        self.action_table.push([Action::None; TOKEN_COUNT]);
    }

    fn get_start_state(&self) -> usize {
        0
    }

    /// Add shifts action to dfa. Takes index of token as input
    fn add_shift_index(&mut self, state: usize, token: usize, next_state: usize) {
        if self.action_table[state][token] != Action::None {
            panic!("Shift conflict");
        }
        self.action_table[state][token] = Action::Shift(next_state);
    }

    fn add_reduce(&mut self, state: usize, rule: &ParserStateRule) {
        for token in rule.lookahead.iter() {
            if self.action_table[state][token.index()] != Action::None {
                panic!("Reduce conflict");
            }
            self.action_table[state][token.index()] = Action::Reduce(rule.index);
        }
    }

    /// Adds goto to dfa, based on index of nonterminal
    fn add_goto_index(&mut self, state: usize, non_terminal: usize, next_state: usize) {
        if self.goto_table[state][non_terminal] != INVALID_SYMBOL {
            panic!("Goto Conflict");
        }
        self.goto_table[state][non_terminal] = next_state;
    }

    pub fn generate(grammar: &Grammar) -> ParserDFA {
        // Using these to make code clearer
        type TokenIndex = usize;
        type NonTerminalIndex = usize;

        let mut dfa = ParserDFA::new();

        // Index in this vec corresponds to index in dfa
        let mut parser_states: Vec<Vec<ParserStateRule>> = Vec::new();

        // Create first state
        let first_state = state_closure(
            vec![ParserStateRule::new(&grammar, 0, vec![])], 
            &grammar
        );
        parser_states.push(first_state);
        dfa.add_state();
        
        let mut i = 0;
        while i < parser_states.len() {
            // i is index of state in dfa
            let state = &parser_states[i];
            

            // Collect all symbols that appear after dot
            // Maps a symbol to the newly created rules for it
            // Uses index of symbol instead of symbol itself
            let mut terminals: HashMap<TokenIndex, Vec<ParserStateRule>> = HashMap::new();
            let mut non_terminals: HashMap<NonTerminalIndex, Vec<ParserStateRule>> = HashMap::new();

            for rule in state.iter() {
                // Check if this is reduce rule
                if rule.dot_pos == rule.rule.rhs.len() {
                    dfa.add_reduce(i, rule);
                    continue;
                }
                
                let after_dot = &rule.rule.rhs[rule.dot_pos];
                match after_dot {
                    Symbol::T(t) => {
                        let index = t.index();
                        terminals
                            .entry(index)
                            .or_insert_with(|| Vec::new())
                            .push(rule.increment(grammar));
                    }
                    Symbol::N(n) => {
                        let index = n.index();
                        non_terminals
                            .entry(index)
                            .or_insert_with(|| Vec::new())
                            .push(rule.increment(grammar));
                    },
                }
            }

            // For each terminal after dot compute new state and add transition
            for (token_index, rules) in terminals.into_iter() {
                let closure = state_closure(rules, grammar);
                
                if let Some(s) = parser_states.iter().position(|s| **s==closure) {
                    // exact same state already exists, just add transition
                    dfa.add_shift_index(i, token_index, s);
                } else {
                    // Add state to dfa
                    parser_states.push(closure);
                    dfa.add_state();
                    let new_state = parser_states.len()-1;
                    dfa.add_shift_index(i, token_index, new_state);
                }
            }

            // For each nonterminal after dot compute new state and add transition
            for (non_terminal_index, rules) in non_terminals.into_iter() {
                let closure = state_closure(rules, grammar);
                
                if let Some(s) = parser_states.iter().position(|s| **s==closure) {
                    // exact same state already exists, just add transition
                    dfa.add_goto_index(i, non_terminal_index, s);
                } else {
                    // Add state to dfa
                    parser_states.push(closure);
                    dfa.add_state();
                    let new_state = parser_states.len()-1;
                    dfa.add_goto_index(i, non_terminal_index, new_state);
                }
            }
            i+=1;
        }

        dfa
    }

    pub fn parse(&self, input: Vec<Node<Token>>, grammar: &Grammar) -> Result<AstNode, String> {
        let mut input: VecDeque<Node<Token>> = input.into_iter().collect();
        let mut current_state = self.get_start_state();
        let mut processed: Vec<(ProcessedSymbol, usize)> = Vec::new();

        while let Some(node) = input.pop_front() {
            let token = node.get();
            let action = self.action_table[current_state][token.index()];
            match action {
                Action::None => return Err(format!("No action for token {:?}, in state {current_state}", token)),
                Action::Shift(next_state) => {
                    processed.push((ProcessedSymbol::T(node), next_state));
                    current_state = next_state;
                },
                Action::Reduce(index) => {
                    let rule = &grammar[index];
                    let non_terminal = rule.lhs;


                    // get last elements of processed
                    let symbols: Vec<ProcessedSymbol> = processed
                        .split_off(processed.len()-rule.rhs.len())
                        .into_iter()
                        .map(|(s, _)| s)
                        .collect();
                    
                    // TO DO: for now we're just taking location of first symbol
                    let loc = symbols[0].get_loc().clone();
                    let new_node = (rule.generator)(symbols);
                    let new_processed = ProcessedSymbol::N(
                        Node::new(
                            (non_terminal ,new_node), 
                            loc
                        )
                    );
                    
                    // Compute new state
                    current_state = match processed.last() {
                        Some(i) => i.1,
                        None => self.get_start_state()
                    };
                    let new_state = self.goto_table[current_state][non_terminal.index()];
                    if new_state == INVALID_SYMBOL {
                        return Err(String::from("No goto action for nonterminal"));
                    }

                    processed.push((new_processed, new_state));
                    current_state=new_state;

                    // If we have just reduced with an EOF, then stop
                    if *node.get() == Token::EOF {
                        break;
                    }

                    // Put processed token back into queue
                    input.push_front(node);
                }
            }
        }
        
        // If all went well we should have only final NonTerminal at end
        if processed.len() != 1 {
            return Err(String::from("Parse error"));
        }
        
        // Extract AstNode from last NonTerminal
        if let Some((ProcessedSymbol::N(n), _)) = processed.pop() {
            let ((_, ast), _) = n.extract(); 
            return Ok(ast); 
        }

        Err(String::from("Parse error"))
    }
}


