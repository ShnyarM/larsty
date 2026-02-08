use std::collections::{VecDeque, HashMap};
use serde::{Serialize, Deserialize};

use super::types::{Symbol, Grammar, Rule, ProcessedSymbol, Associativity};
use super::rules::{NonTerminal, AstNode};
use crate::ast::Node;
use crate::token::Token;

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

/// This struct gets used during the construction of the dfa, get turned to a ParserDFA at the end
struct ConstructingParserDFA {
    state_count: usize,
    goto_table: Vec<[usize; NonTerminal::COUNT]>,
    // (Action, precedence, Associativity)
    action_table: Vec<[(Action, usize, Associativity); Token::COUNT]>
}

#[derive(Debug)]
pub struct ParserDFA {
    state_count: usize,
    goto_table: Vec<[usize; NonTerminal::COUNT]>,
    action_table: Vec<[Action; Token::COUNT]>,
}

#[derive(Serialize, Deserialize)]
pub struct CompressedParserDFA {
    state_count: usize,
    goto_table: Vec<HashMap<usize, usize>>,
    action_table: Vec<HashMap<usize, Action>>
}

// TO DO: check if caching works
// TO DO: Make empty rules work
fn first(symbol: &Symbol, grammar: &Grammar, first_cache: &mut Vec<(NonTerminal, Vec<Token>)>) -> Vec<Token> {
    match symbol {
        Symbol::T(t) => vec![t.clone()],
        Symbol::N(n) => {
            // Check if we can find in cache
            for (non_terminal, list) in first_cache.iter() {
                if *non_terminal == *n {
                    println!("Avoided recomp");
                    return list.clone();
                }
            }
            println!("Didnt Avoid recomp");

            let mut result = Vec::new();
            for r in grammar.rules.iter() {
                if r.lhs == *n {
                    // If we have a panic here this means we have an epsilon rule, not yet
                    // supported
                    result.append(&mut first(&r.rhs[0], grammar, first_cache));
                }
            } 

            // Add to cache
            first_cache.push((*n, result.clone()));
            result
        }
    }
}

/// Compute the closure for a set of rules
fn state_closure<'a>(
    state_rules: Vec<ParserStateRule<'a>>, 
    grammar: &'a Grammar,
    first_cache: &mut Vec<(NonTerminal, Vec<Token>)>
) -> Vec<ParserStateRule<'a>> {
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
                lookahead.append(&mut first(s, grammar, first_cache));
            } else { 
                // otherwise use already existing lookahead
                lookahead = r.lookahead.clone();
            }

            for (i, g) in grammar.rules.iter().enumerate() {
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
            rule: &grammar.rules[index],
            lookahead,
            dot_pos: 0
        }
    }

    /// Returns copy of rule, but with the dot moved by one spot
    fn increment<'t>(&self, grammar: &'t Grammar) -> ParserStateRule<'t> {
        ParserStateRule { 
            index: self.index, 
            rule: &grammar.rules[self.index], 
            lookahead: self.lookahead.clone(), 
            dot_pos: self.dot_pos+1 
        }
    }

    /// Checks if two state rules have same core, i.e. everything same except for lookahead
    fn eq_core(&self, other: &Self) -> bool {
        self.index == other.index && self.rule == other.rule && self.dot_pos == other.dot_pos
    }
}

impl ConstructingParserDFA {
    fn new() -> ConstructingParserDFA {
        ConstructingParserDFA { 
            state_count: 0, 
            goto_table: Vec::new(), 
            action_table: Vec::new() 
        }
    }

    fn add_state(&mut self) {
        self.state_count += 1;
        self.goto_table.push([INVALID_SYMBOL; NonTerminal::COUNT]);
        self.action_table.push([(Action::None, 0, Associativity::NonAssoc); Token::COUNT]);
    }

    /// Takes in new action for specific lookahead and state and decides to keep new or old one
    fn solve_conflict(&mut self, state:usize, token: usize, new_action:(Action, usize, Associativity)) {
        let (cur_action, cur_precedence, _) = self.action_table[state][token];
        let (new_action, new_precedence, new_assoc) = new_action;

        if cur_action == Action::None {
            self.action_table[state][token] = (
                new_action,
                new_precedence,
                new_assoc
            );
            return;
        }

        // We have conflict, resolve
        
        // Dont do anything if precedence is lower
        if cur_precedence > new_precedence {
            return;
        }

        if cur_precedence < new_precedence {
            self.action_table[state][token] = (
                new_action,
                new_precedence,
                new_assoc
            );  
            return;
        }

        // Same precedence, solve with associativity 
        match new_assoc {
            // We take reduce if associativity is left
            Associativity::Left => {
                if let Action::Shift(_) = cur_action {
                    self.action_table[state][token] = (
                        new_action,
                        new_precedence,
                        new_assoc
                    );
                }
            },
            // We take shift if associativity is right
            Associativity::Right => {
                if let Action::Reduce(_) = cur_action {
                    self.action_table[state][token] = (
                        new_action,
                        new_precedence,
                        new_assoc
                    );
                }
            }
            Associativity::NonAssoc => panic!("Shift/Reduce conflict, caused by NonAssoc"),
        }
    }

    /// Add shifts action to dfa. Takes index of token as input
    /// Uses precedence and associativity to solve conflicts
    fn add_shift_index(&mut self, state: usize, token: usize, next_state: usize, grammar: &Grammar) {
        let new_action = Action::Shift(next_state);
        let new_precedence = grammar.get_precedence_index(token);
        let new_assoc = grammar.get_assoc_index(token);

        self.solve_conflict(state, token, (new_action, new_precedence, new_assoc));
    }

    fn add_reduce(&mut self, state: usize, rule: &ParserStateRule, grammar: &Grammar) {
        for token in rule.lookahead.iter() {
            let (precedence, assoc) = grammar.get_rule_precedence(rule.index);
            let action = Action::Reduce(rule.index);
            self.solve_conflict(state, token.index(), (action, precedence, assoc));
        }
    }

    /// Adds goto to dfa, based on index of nonterminal
    fn add_goto_index(&mut self, state: usize, non_terminal: usize, next_state: usize) {
        if self.goto_table[state][non_terminal] != INVALID_SYMBOL {
            panic!("Goto Conflict");
        }
        self.goto_table[state][non_terminal] = next_state;
    }
}

impl ParserDFA {
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
            goto_table.push([INVALID_SYMBOL; NonTerminal::COUNT]);
            for (symbol, s2) in t.iter() {
                goto_table[i][*symbol] = *s2;
            } 
        }

        // Copy actions
        let mut action_table = Vec::new();
        for (i, t) in compressed.action_table.iter().enumerate() {
            action_table.push([Action::None; Token::COUNT]);
            for (symbol, s2) in t.iter() {
                action_table[i][*symbol] = *s2;
            } 
        }

        ParserDFA { state_count: compressed.state_count, goto_table, action_table }
    }

    fn from_constructing(dfa: ConstructingParserDFA) -> ParserDFA {
        let action_table = dfa.action_table
            .into_iter()
            .map(|l| {
                std::array::from_fn(|i| l[i].0)
            })
            .collect();

        ParserDFA { state_count: dfa.state_count, goto_table: dfa.goto_table, action_table }
    }
    
    fn get_start_state(&self) -> usize {
        0
    }

    pub fn generate(grammar: &Grammar) -> ParserDFA {
        // Using these to make code clearer
        type TokenIndex = usize;
        type NonTerminalIndex = usize;

        let mut dfa = ConstructingParserDFA::new();
        
        // This caches the results of the first() function
        let mut first_cache = Vec::new();

        // Index in this vec corresponds to index in dfa
        let mut parser_states: Vec<Vec<ParserStateRule>> = Vec::new();

        // Create first state
        let first_state = state_closure(
            vec![ParserStateRule::new(&grammar, 0, vec![])], 
            &grammar,
            &mut first_cache
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
                    dfa.add_reduce(i, rule, grammar);
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
                let closure = state_closure(rules, grammar, &mut first_cache);
                
                if let Some(s) = parser_states.iter().position(|s| **s==closure) {
                    // exact same state already exists, just add transition
                    dfa.add_shift_index(i, token_index, s, grammar);
                } else {
                    // Add state to dfa
                    parser_states.push(closure);
                    dfa.add_state();
                    let new_state = parser_states.len()-1;
                    dfa.add_shift_index(i, token_index, new_state, grammar);
                }
            }

            // For each nonterminal after dot compute new state and add transition
            for (non_terminal_index, rules) in non_terminals.into_iter() {
                let closure = state_closure(rules, grammar, &mut first_cache);
                
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
        
        // TO DO: LALR(1)
        ParserDFA::from_constructing(dfa)
    }

    pub fn parse(&self, input: Vec<Node<Token>>, grammar: &Grammar) -> Result<AstNode, String> {
        let mut input: VecDeque<Node<Token>> = input.into_iter().collect();
        let mut current_state = self.get_start_state();
        let mut processed: Vec<(ProcessedSymbol, usize)> = Vec::new();

        while let Some(node) = input.pop_front() {
            let token = node.get();
            println!("{:?}", token);
            let action = self.action_table[current_state][token.index()];
            match action {
                Action::None => return Err(format!("No action for token {:?}, in state {current_state}", token)),
                Action::Shift(next_state) => {
                    processed.push((ProcessedSymbol::T(node), next_state));
                    current_state = next_state;
                },
                Action::Reduce(index) => {
                    println!("Reduce, index {index}");
                    let rule = &grammar.rules[index];
                    let non_terminal = rule.lhs;

                    // get last elements of processed
                    let symbols: Vec<ProcessedSymbol> = processed
                        .split_off(processed.len()-rule.rhs.len())
                        .into_iter()
                        .map(|(s, _)| s)
                        .collect();
                    
                    let new_processed = ProcessedSymbol::N(
                        non_terminal,
                        (rule.generator)(symbols)
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

                    // Put processed token back into queue
                    input.push_front(node);
                }
            }
        }
        
        // If all went well we should have only final NonTerminal with EOF at end
        if processed.len() != 2 {
            println!("Count: {}, {:?}", processed.len(), processed);
            return Err(String::from("Parse error, more than one processed Symbol at the end"));
        }

        processed.pop();
        // Extract AstNode from last NonTerminal
        if let Some((ProcessedSymbol::N(_, ast), _)) = processed.pop() {
            return Ok(ast); 
        }

        Err(String::from("Parse error"))
    }
}


