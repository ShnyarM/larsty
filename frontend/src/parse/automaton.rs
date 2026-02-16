use std::collections::{VecDeque, HashMap};
use std::mem;
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

/// The core of a ParserStateRule, i.e. the rule without the lookahead. Used for LALR(1) parser
/// creation
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
struct ParserStateRuleCore {
    pub index: usize, // Index of rule in grammar
    pub dot_pos: usize, // 0 represents right at the beginning. Tells us which symbol is in front
}

/// Represents an action in the action table of the Parser
#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
enum Action {
    None,
    Shift(usize),
    Reduce(usize), // Index of rule to be applied
}

/// This struct gets used during the construction of the dfa, gets turned to a ParserDFA at the end
/// The only difference is that the action table also stores precedence and associativity of the
/// action (derived from token). This information is only needed during construction.
struct ConstructingParserDFA {
    state_count: usize,
    goto_table: Vec<[usize; NonTerminal::COUNT]>,
    // (Action, precedence, Associativity)
    action_table: Vec<[(Action, usize, Associativity); Token::COUNT]>
}

/// Actual ParserDFA that gets used during runtime when parsing the Token Stream.
#[derive(Debug)]
pub struct ParserDFA {
    state_count: usize,
    goto_table: Vec<[usize; NonTerminal::COUNT]>,
    action_table: Vec<[Action; Token::COUNT]>,
}

/// An compressed version of ParserDFA, that only stores relevant table entries and doesnt store
/// entries that are empty. This version gets stored into the binary.
#[derive(Serialize, Deserialize)]
pub struct CompressedParserDFA {
    state_count: usize,
    goto_table: Vec<HashMap<usize, usize>>,
    action_table: Vec<HashMap<usize, Action>>
}

// TO DO: check if caching works
/// Returns all tokens that can come first when using the symbol.
/// Second return value specifies if symbol can derive epsilon (i.e. empty terminal)
/// Third argument is a cache that contains cached results of previous calls. Otherwise we would
/// have to keep recomputing for NonTerminals that have already been calculated.
/// Cache gets filled automatically
fn first(
    symbol: &Symbol, 
    grammar: &Grammar, 
    cache: &mut Vec<(NonTerminal, (Vec<Token>, bool))>
) -> (Vec<Token>, bool) {
    match symbol {
        Symbol::T(t) => (vec![t.clone()], false),
        Symbol::N(n) => {
            // Check if we can find in cache
            for (non_terminal, (list, epsilon)) in cache.iter() {
                if *non_terminal == *n {
                    println!("Avoided recomp");
                    return (list.clone(), *epsilon);
                }
            }
            println!("Didnt Avoid recomp");
            println!("Computing {:?}", n);

            let mut result = Vec::new();
            let mut can_derive_epsilon = false;

            for r in grammar.rules.iter() {
                //Search for rules deriving this nonterminal
                if r.lhs == *n {
                    // variable that tells us if this rule can derive epsilon
                    let mut rule_epsilon = true;

                    // Go through all symbols until we find one that can't derive epsilon
                    for symbol in r.rhs.iter() {
                        let (mut new_tokens, epsilon) = first(&symbol, grammar, cache);
                        result.append(&mut new_tokens);

                        if !epsilon {
                            rule_epsilon = false;
                            break;
                        }
                    }
                    if rule_epsilon {
                        can_derive_epsilon = true;
                    }
                }
            } 

            // If we can derive epsilon check all symbols that can come after

            // Add to cache
            cache.push((*n, (result.clone(), can_derive_epsilon)));
            (result, can_derive_epsilon)
        }
    }
}

/// Compute the closure for a set of rules
/// Third argument is cache that is used for the first() function.
/// If no cache is present just use an empty vec
fn state_closure<'a>(
    state_rules: Vec<ParserStateRule<'a>>, 
    grammar: &'a Grammar,
    first_cache: &mut Vec<(NonTerminal, (Vec<Token>, bool))>
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
            let mut can_derive_epsilon = true;
            
            // We go through all symbols after dot until we find one that can't derive epsilon.
            // At that point we stop
            let mut i = 1;
            while let Some(s) = &r.rule.rhs.get((r.dot_pos+i) as usize) {
                let (mut new_lookahead, epsilon) = first(s, grammar, first_cache);
                lookahead.append(&mut new_lookahead);

                if !epsilon {
                    can_derive_epsilon = false;
                    break;
                }
                i+=1;
            }  

            // if we can derive epsilon, copy lookahead of already existing lookahead from rule 
            if can_derive_epsilon {
                lookahead = r.lookahead.clone();
            }
            
            // Now we search for all rules with this nonterminal and add their rule to state
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
    
    /// Returns only the core of the rule. Used for creation of LALR(1) parser
    fn core(&self) -> ParserStateRuleCore {
        ParserStateRuleCore {
            index: self.index,
            dot_pos: self.dot_pos 
        }
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

    fn add_state(&mut self) -> usize {
        self.state_count += 1;
        self.goto_table.push([INVALID_SYMBOL; NonTerminal::COUNT]);
        self.action_table.push([(Action::None, 0, Associativity::NonAssoc); Token::COUNT]);
        self.state_count-1
    }

    /// Takes in new action for specific lookahead and state and decides to keep new or old one
    fn solve_conflict(
        &mut self,
        state:usize,
        token: usize,
        new_action:(Action, usize, Associativity)) 
    {
        let (cur_action, cur_precedence, _) = self.action_table[state][token];
        let (new_action, new_precedence, new_assoc) = new_action;

        if cur_action == new_action {
            return;
        }
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
    fn add_shift_index(
        &mut self,
        state: usize,
        token: usize,
        next_state: usize,
        grammar: &Grammar) 
    {
        let new_action = Action::Shift(next_state);
        let new_precedence = grammar.get_precedence_index(token);
        let new_assoc = grammar.get_assoc_index(token);

        self.solve_conflict(state, token, (new_action, new_precedence, new_assoc));
    }

    fn add_reduce(&mut self, state: usize, rule: &ParserStateRule, grammar: &Grammar) {
        for token in rule.lookahead.iter() {
            let (precedence, assoc) = grammar
                .get_rule_precedence(rule.index)
                .unwrap_or((0, Associativity::NonAssoc));

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
    pub fn new() -> ParserDFA {
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

    fn set_state_count(&mut self, new_count: usize) {
        self.state_count = new_count;
        self.goto_table.resize(new_count, [INVALID_SYMBOL; NonTerminal::COUNT]);
        self.action_table.resize(new_count, [Action::None; Token::COUNT]);
    }
    
    fn get_start_state(&self) -> usize {
        0
    }
    
    /// Converts the ParserDFA to a LALR(1) parser, given that self is currently a LR(1) parser.
    /// Requires a list Vec<Vec<ParserStateRule>> that map each state of the parser to the set of
    /// ParserStateRules it represents
    fn to_lalr(
        self, 
        states: Vec<Vec<ParserStateRule>>
    ) -> ParserDFA {
        let mut new_dfa = ParserDFA::new();
            
        // Map a set of RuleCores to all states indices that belong to that
        let mut equal_states: HashMap<Vec<ParserStateRuleCore>, Vec<usize>> = HashMap::new();
        
        // this part only workd because the Vec<ParserStateRule> have already been sorted when they
        // were created in state_closure()
        for (i, state) in states.iter().enumerate() {
            let cores = state.iter().map(|r| r.core()).collect();
            let e = equal_states.entry(cores).or_insert_with(|| Vec::new());
            e.push(i);
        }
        
        // list of the new states. Each element shows all old states that are present in new one
        let mut equal_states_list: Vec<Vec<usize>> = equal_states
            .into_iter()
            .map(|(_, l)| l)
            .collect();

        // reorder states so that partition that contains start is at beginning
        let start_state = self.get_start_state();
        if let Some(pos) = equal_states_list.iter().position(|v| v.contains(&start_state)) {
            if pos != start_state {
                equal_states_list.swap(start_state, pos);
            }
        }
        
        // create mapping from old state to new state
        let mut old_to_new: Vec<usize> = Vec::new();
        old_to_new.resize(states.len(), 0);

        for (i, new_state) in equal_states_list.iter().enumerate() {
            for old_state in new_state {
                old_to_new[*old_state] = i;
            }
        }
            
        new_dfa.set_state_count(equal_states_list.len());
        // Use compressed representation, more efficient
        let compressed = self.compress();

        // go through all actions and add back
        for (old_state, actions) in compressed.action_table.iter().enumerate() {
            let new_state = old_to_new[old_state];
            for (token_index, action) in actions.iter() {
                let mut new_action = *action;
                
                // update state we shift to if shift
                if let Action::Shift(s) = new_action {
                    new_action = Action::Shift(old_to_new[s])
                }

                // Check for newly created conflict
                let present_action = new_dfa.action_table[new_state][*token_index];
                if present_action != Action::None && present_action != new_action {
                    panic!("Couldn't convert to LALR(1), as given grammar is LR(1), but not LALR(1)");
                }

                new_dfa.action_table[new_state][*token_index] = new_action;
            }
        }

        // go through all gotos and add back
        for (old_state, gotos) in compressed.goto_table.iter().enumerate() {
            let new_state = old_to_new[old_state];
            for (non_terminal_index, goto_state) in gotos.iter() {
                new_dfa.goto_table[new_state][*non_terminal_index] = old_to_new[*goto_state];
            }
        }

        println!("Parser State count of LR(1) parser: {}", states.len());
        println!("Parser State count after converting to LALR(1): {}", new_dfa.state_count);
        new_dfa
    }

    /// generates a LALR(1) parser based on the given grammar
    pub fn generate(grammar: &Grammar) -> ParserDFA {
        // This function first creates a LR(1) parser and then converts it to a LALR(1) parser at
        // the end

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
                    let new_state = dfa.add_state();
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
                    let new_state = dfa.add_state();
                    dfa.add_goto_index(i, non_terminal_index, new_state);
                }
            }
            i+=1;
        }
        
        ParserDFA::from_constructing(dfa).to_lalr(parser_states)
    }
    
    /// Parses a given TokenStream to an AST. Requires the grammar to have access to the generators
    /// that produce the AST nodes.
    pub fn parse(&self, input: Vec<Node<Token>>, grammar: &Grammar) -> Result<AstNode, String> {
        let mut input = input;
        let mut current_state = self.get_start_state();
        let mut processed: Vec<(ProcessedSymbol, usize)> = Vec::new();
        
        let mut index = 0;
        while index < input.len() {
            let node = &input[index];
            let token = node.get();
            println!("{:?}", token);
            let action = self.action_table[current_state][token.index()];
            match action {
                Action::None => return Err(
                    format!("No action for token {:?}, in state {current_state}",token)
                ),
                Action::Shift(next_state) => {
                    // take ownership of node, replace value with garbage data
                    let node = mem::replace(&mut input[index], Node::no_loc(Token::EOF));

                    processed.push((ProcessedSymbol::T(node), next_state));
                    current_state = next_state;

                    // advance character
                    index += 1;
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
                }
            }
        }
        
        // If all went well we should have only final NonTerminal with EOF at end
        if processed.len() != 2 {
            println!("Count: {}, {:?}", processed.len(), processed);
            return Err(String::from(
                "Parse error, more than one processed Symbol at the end. Perhaps the start rule at the beginning of the grammar rules is missing."
            ));
        }

        processed.pop(); // Remove EOF at end
        
        // Extract AstNode from last NonTerminal
        if let Some((ProcessedSymbol::N(_, ast), _)) = processed.pop() {
            return Ok(ast); 
        }

        Err(String::from("Parse error. Input is potentially faulty"))
    }
}
