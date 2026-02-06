use std::{cmp::max, collections::{HashMap, HashSet, VecDeque}};

use serde::{Serialize, Deserialize};

use crate::{Location, Node};

pub const EPSILON_CHAR: u8 = 0;
const ALPHABET_SIZE: usize = 256;
const INVALID_SYMBOL: usize = usize::MAX;
pub const NOT_ACCEPT:i32 = -1;

#[derive(Debug)]
pub struct DFA{
    state_count: usize,
    transitions: Vec<[usize; ALPHABET_SIZE]>,
    // Stores the information of which states are accept states and the associated action. If
    // NOT_ACCEPT,then not an accept state, otherwise it is the action
    actions: Vec<i32>
}

/// A compressed representation of DFA. Is meant to be used for storing the DFA, not for actual
/// execution
#[derive(Serialize, Deserialize)]
pub struct CompressedDFA{
    state_count: usize,
    transitions: Vec<HashMap<u8, usize>>,
    actions: HashMap<usize, i32>
}

#[derive(Debug)]
pub struct NFA{
    state_count: usize,
    transitions: Vec<HashMap<u8, Vec<usize>>>,
    // Stores the information of which states are accept states and the associated action. If
    // NOT_ACCEPT,then not an accept state, otherwise it is the action
    actions: Vec<i32>
}

impl DFA{
    pub fn new() -> DFA {
        let mut dfa = DFA {
            state_count: 1,
            transitions: Vec::new(),
            actions: Vec::new()
        };
        dfa.adjust_vec();
        dfa
    }

    pub fn compress(&self) -> CompressedDFA {
        // Copy transitions
        let mut transitions = Vec::new();
        for t in self.transitions.iter() {
            let mut map = HashMap::new();
            for (symbol, s2) in t.iter().enumerate() {
                if *s2 != INVALID_SYMBOL {
                   map.insert(symbol as u8, *s2); 
                }
            } 
            transitions.push(map);
        }

        // Copy actions
        let mut actions = HashMap::new();
        for (s, a) in self.actions.iter().enumerate() {
            if *a != NOT_ACCEPT {
                actions.insert(s, *a);
            }
        }

        CompressedDFA { state_count: self.state_count, transitions, actions }
    }
    
    pub fn from_compressed(compressed: CompressedDFA) -> DFA {
        // Copy transitions
        let mut transitions = Vec::new();
        for (i, t) in compressed.transitions.iter().enumerate() {
            transitions.push([INVALID_SYMBOL; ALPHABET_SIZE]);
            for (symbol, s2) in t.iter() {
                transitions[i][*symbol as usize] = *s2;
            } 
        }

        // Copy actions
        let mut actions = Vec::new();
        actions.resize(compressed.state_count, NOT_ACCEPT);
        for (s, a) in compressed.actions.iter() {
            actions[*s] = *a;
        }

        DFA { state_count: compressed.state_count, transitions, actions }
    }

    // Makes sure that the vector of transitions matches the length of all states
    fn adjust_vec(&mut self) {
        if self.transitions.len() < self.state_count {
            self.transitions.resize(self.state_count, [INVALID_SYMBOL; ALPHABET_SIZE]);
            self.actions.resize(self.state_count, NOT_ACCEPT);
        }
    }

    pub fn add_transition(&mut self, from:usize, to:usize, symbol:u8) {
        self.transitions[from][symbol as usize] = to;
    }

    // Add this incase later start state is defined by something else
    pub fn get_start_state(&self) -> usize {
        0
    }

    pub fn get_state_count(&self) -> usize {
        self.state_count
    }

    /// Set an action for a state. Can make a state an accept state
    pub fn set_action(&mut self, s:usize, action:i32) {
        self.actions[s] = action;
    }

    /// Returns the amount of different actions present in this DFA
    fn action_count(&self) -> i32 {
        let mut highest_seen = -1;

        for a in self.actions.iter() {
            if *a > highest_seen {
                highest_seen = *a;
            }
        }
        highest_seen+2
    }

    /// Returns the newly created state
    pub fn add_state(&mut self, action:i32) -> usize {
        let state = self.state_count;
        self.state_count += 1;

        self.transitions.push([INVALID_SYMBOL; ALPHABET_SIZE]);
        self.actions.push(action);

        state
    }
    
    fn set_state_count(&mut self, count: usize) {
        self.state_count = count;
        self.adjust_vec();
    }

    /// Changes the DFA to be the minimal equivalent DFA
    fn minimize(&mut self) {
        // Algorithm works by creating partitions of the states and iteratively refining them.
        // A pair of states is indistuingishable if for every symbol they go to same partition of
        // state.
        // Essentially, a pair of states is indistinguishable if they in practice are the same
        // state.
        // We hold the invariant that two indistinguishable are always in the same partition.
        // We start with the partition based on the actions of the states. We then iteratively
        // refine the partitions by splitting sets of states if they have a different partition for
        // a specific symbol.
        // If no change  can be made anymore, were done. Each partition then represents a single
        // state.
        let mut partitions: Vec<Vec<usize>> = Vec::new();
        let mut state_to_partition: Vec<usize> = Vec::new();
        partitions.resize(self.action_count() as usize, Vec::new());
        state_to_partition.resize(self.get_state_count(), 0);
        
        // Create initial partition based on action of state
        for (i, a) in self.actions.iter().enumerate() {
            partitions[(*a+1) as usize].push(i);
            state_to_partition[i] = *a as usize;
        }

        // Use transitions of compressed DFA, more effective here
        let transitions = self.compress().transitions;

        // Refine partitions until fixpoint reached
        let mut change_occured = true;
        while change_occured {
            change_occured = false;
            let mut new_partitions: Vec<Vec<usize>> = Vec::new();

            // Go through all partitions
            for p in partitions.into_iter() {
                // For a given set of transitions, store all states that have exactly the same
                // transitions
                let mut signatures: HashMap<Vec<(u8, usize)>, Vec<usize>> = HashMap::new();

                for s in p.iter() {
                    // Create a sorted list of all transitions to then insert into hashmap
                    // We mutate transitions to point to partition it goes to check if
                    // distinguishable 
                    let mut transition_set: Vec<(u8, usize)> = transitions[*s]
                        .iter()
                        .map(|(k,v)| (*k, state_to_partition[*v]))
                        .collect();
                    transition_set.sort();

                    let signature_list = signatures.entry(transition_set).or_insert(Vec::new());
                    signature_list.push(*s);
                }

                // Check if different set of transitions exist
                if signatures.len() > 1 {
                    change_occured = true;

                    // For each group create a new partition
                    for (_, states) in signatures.into_iter() {
                        let index = new_partitions.len();
                        for s in states.iter() {
                            state_to_partition[*s] = index;
                        } 
                        new_partitions.push(states);
                    }
                } else {
                    let index = new_partitions.len();
                    for s in p.iter() {
                        state_to_partition[*s] = index;
                    }
                    new_partitions.push(p);
                }
            }  
            partitions = new_partitions;
        }

        // Construct new DFA
        // Each partition corresponds to a new state in minimized DFA
        
        // Remove empty partitions
        partitions = partitions.into_iter().filter(|p| p.len() > 0).collect();

        let mut new_dfa = DFA::new();
        new_dfa.set_state_count(partitions.len()); 

        // reorder partition so that partition that contains start is at beginning
        if let Some(pos) = partitions.iter().position(|v| v.contains(&0)) {
            if pos != 0 {
                // update state to partition mapping
                for s in partitions[0].iter() {
                    state_to_partition[*s] = pos;
                }
                for s in partitions[pos].iter() {
                    state_to_partition[*s] = 0;
                }
                partitions.swap(0, pos);
            }
        }

        for (i, p) in partitions.into_iter().enumerate() {
            // Only 1 state needs to be used, since they all have same transitions
            new_dfa.actions[i] = self.actions[p[0]];
            for (symbol, s) in transitions[p[0]].iter() {
                new_dfa.transitions[i][*symbol as usize] = state_to_partition[*s] as usize;
            }
        }

        *self = new_dfa;
    }

    /// Performs standard execution of an input
    pub fn execute(&self, input: &str) -> bool {
        let mut s = self.get_start_state();

        for symb in input.bytes() {
            s = self.transitions[s][symb as usize];
            
            if s == INVALID_SYMBOL {
                return false;
            } 
        }

        self.actions[s] != NOT_ACCEPT 
    }

    /// Performs longest matching on the input. Returns the recognized tokens as a list of matched
    /// actions.
    /// Returns Err when there was a lexing error.
    pub fn longest_matching<'a>(&self, input:&'a str) -> Result<Vec<Node<(i32, &'a str)>>, &str> {
        let mut line = 1;
        let mut line_char = 0;

        let mut s = self.get_start_state();
        let mut actions = Vec::new();

        let bytes = input.as_bytes();
        let mut i = 0;

        let mut last_accept_action = -1;
        let mut last_accept_byte = 0;
        let mut last_accept_line = 0;
        let mut last_accept_line_char = 0;
        let mut last_consumed_byte = 0;

        while i < bytes.len() {
            let b = bytes[i];
            if b == b'\n' {
                i += 1;
                line += 1;
                line_char = 0;
                continue;
            }

            let next_state = self.transitions[s][bytes[i] as usize];

            if next_state != INVALID_SYMBOL {
                s = next_state;
                i += 1;
                line_char += 1;

                if self.actions[s] != NOT_ACCEPT {
                    last_accept_byte = i;
                    last_accept_action = self.actions[s];
                    last_accept_line = line;
                    last_accept_line_char = line_char;
                }
                continue;
            }

            // We can't continue. Check if we have been in accept state
            if last_accept_action != -1 {
                // Consume until this point
                let detected = &input[last_consumed_byte as usize..last_accept_byte as usize];

                let location = Location::new(last_accept_line, last_accept_line_char);
                actions.push(Node::new((last_accept_action, detected), location));
                last_consumed_byte = last_accept_byte;

                // Return state
                s = self.get_start_state();
                i = last_accept_byte as usize;
                last_accept_action = -1;
                last_accept_byte = 0;
                continue;
            }

            // We have lexing error
            return Err("Lexing error");
        }

        // At end we check if we're at an accept state
        if self.actions[s] == NOT_ACCEPT {
            return Err("Lexing error");
        }

        // consume remaining
        let detected = &input[last_consumed_byte as usize..last_accept_byte as usize];

        let location = Location::new(last_accept_line, last_accept_line_char);
        actions.push(Node::new((last_accept_action, detected), location));

        Ok(actions)
    }
}

impl NFA{
    /// Get a NFA with a single state that is both start and accept
    pub fn new() -> NFA {
        let mut n = NFA { 
            state_count: 1, 
            transitions: Vec::new(),
            actions: Vec::new() 
        };
        n.adjust_vec();
        n
    }
    
    // Makes sure that the vector of transitions matches the length of all states
    fn adjust_vec(&mut self) {
        if self.transitions.len() < self.state_count {
            self.transitions.resize(self.state_count, HashMap::new());
            self.actions.resize(self.state_count, NOT_ACCEPT);
        }
    }

    /// Get the HashMap of all transitions for a state
    fn get_all_transitions(&self, state:usize) -> &HashMap<u8, Vec<usize>> {
        &self.transitions[state]
    }

    /// Get mutable list of transitions for a state and a symbol
    fn get_transitions_mut(&mut self, from:usize, symbol:u8) -> &mut Vec<usize> {
        // Creates entry if it doesn't exist
        self.transitions[from].entry(symbol).or_insert_with(|| Vec::new()) 
    }

    /// Get list of transitions for a state and a symbol
    /// Returns None if no transitions exist for that state and symbol
    fn get_transitions(&self, from:usize, symbol:u8) -> Option<&Vec<usize>> {
        self.transitions[from].get(&symbol) 
    }

    pub fn add_transition(&mut self, from:usize, to:usize, symbol:u8) {
        let set = self.get_transitions_mut(from, symbol);
        set.push(to);
    }

    /// Returns a list of all accepting states
    pub fn get_accept_states(&self) -> Vec<usize> {
        let mut result = vec![];
        for (s, a) in self.actions.iter().enumerate() {
            if *a != NOT_ACCEPT {
                result.push(s);
            }
        }

        result
    }

    // Add this incase later start state is defined by something else
    pub fn get_start_state(&self) -> usize {
        0
    }

    pub fn get_state_count(&self) -> usize {
        self.state_count
    }

    /// Set action for a state. Can be used to make set accepting
    pub fn set_action(&mut self, s:usize, action:i32) {
        self.actions[s] = action;
    }

    pub fn set_state_count(&mut self, new_count:usize) {
        self.state_count = new_count;
        self.adjust_vec();
    }

    /// Merges two NFAs by adding all states of other to this NFA
    /// 
    /// New accept state depends on accept argument. If 0, the accept state of self is taken. If 1,
    /// the accept state of other is taken
    ///
    /// new_transitions is a list of transitions to be added during this merge, allowing for
    /// connections between self and other to be made.
    /// Each entry in new_transitions is of the format:
    ///
    /// ((nfa, state1), (nfa, state2), symbol)
    /// nfa defines which NFA we're referring to. 0 indicates self, 1 indicates other.
    /// Then a transition from state1 to state2 is made with symbol being the character.
    ///
    /// Ex.: ((0, 5), (1, 6), 'c') Creates a transition from self's state 5 to others state 6 for
    /// the symbol 'c'.
    pub fn merge(&mut self, other:&NFA, new_transitions:&Vec<((i32, usize), (i32, usize), u8)>) -> &mut NFA{
        let orig_count = self.get_state_count();
        self.set_state_count(orig_count+other.get_state_count());

        // Copy transitions
        for (state1, hashmap) in other.transitions.iter().enumerate() {
            for (symb, state_list) in hashmap.iter() {
                for state2 in state_list {
                    self.add_transition(state1+orig_count, state2+orig_count, symb.clone());   
                } 
            } 
        }

        // Copy accept states
        for (i, action) in other.actions.iter().enumerate() {
            self.actions[i+orig_count] = *action;
        }

        // add connections
        for ((nfa1, state1), (nfa2, state2), symbol) in new_transitions {
            // If we take state from self, nfa1/2 will be 0
            self.add_transition(
                state1+orig_count*(*nfa1 as usize),
                state2+orig_count*(*nfa2 as usize),
                *symbol
            );
        }
              
        self
    }

    /// For a set of states, compute all states that can be reached through epsilon moves
    /// Returns the list sorted
    fn epsilon_closure(&self, states:&Vec<usize>) -> Vec<usize> {
        // For each state we check which states we can reach through an epsilon transition.
        // We do this until we reach a fixpoint
        let mut seen_states: HashSet<usize> = HashSet::new();
        let mut to_process = VecDeque::new();

        for s in states {
            seen_states.insert(*s);
            to_process.push_back(*s);
        }

        // Compute until nothing in queue
        while let Some(s) = to_process.pop_front() {
            if let Some(reached_states) = self.get_transitions(s, EPSILON_CHAR){
                for x in reached_states {
                    if seen_states.insert(*x) {
                        to_process.push_back(*x);
                    }
                }
            }
        }

        let mut result: Vec<usize> = seen_states.into_iter().collect();
        result.sort();
        result
    }

    pub fn to_dfa(&self) -> DFA {
        // Map each new state in DFA to a set of states
        let mut new_states: Vec<Vec<usize>> = Vec::new();

        // Keep track of all set of states we have inserted with their state in DFA
        let mut seen_sets: HashMap<Vec<usize>, usize> = HashMap::new();
        let mut dfa = DFA::new();

        let start_closure = self.epsilon_closure(&vec![self.get_start_state()]);
        seen_sets.insert(start_closure.clone(), dfa.get_start_state());
        new_states.push(start_closure);

        // Go through each new state and add new states based on transitions.
        // New states are set of states from NFA. new_states keeps track of what state represents which
        // set of states
        let mut i = 0;
        while i < new_states.len() {
            let set = &new_states[i];
            let mut highest_action = -1;

            // We create a new HashMap and contains all possible transitions for this set of states
            let mut transitions: HashMap<u8, Vec<usize>> = HashMap::new();
            for s in set {
                let t = self.get_all_transitions(*s);
                for (symbol, list) in t.iter() {
                    // Ignore epsilon transitions
                    if *symbol == EPSILON_CHAR { continue; } 

                    // We have already added this symbol to our new list
                    if let Some(existing) = transitions.get_mut(symbol) {
                        existing.extend(list.clone());
                    }else{ // Create the entry for this list
                        transitions.insert(*symbol, list.clone());
                    }
                }
                
                let state_action = self.actions[*s];
                highest_action = max(highest_action, state_action);
            }

            // For each symbol in transition table we create new state
            for (symbol, list) in transitions {
                let closure = self.epsilon_closure(&list);

                if let Some(existing_state) = seen_sets.get(&closure) {
                    dfa.add_transition(i, *existing_state, symbol);
                }else{
                    // Action will be adjusted when processing
                    let new_state = dfa.add_state(NOT_ACCEPT);
                    dfa.add_transition(i, new_state, symbol);

                    seen_sets.insert(closure.clone(), new_state);
                    new_states.push(closure);
                }
            }

            dfa.set_action(i, highest_action);

            i+=1;
        }
        
        println!("State count before minimizing DFA: {}", dfa.get_state_count());
        dfa.minimize();
        println!("State count after minimizing DFA: {}", dfa.get_state_count());
        dfa
    }
}
