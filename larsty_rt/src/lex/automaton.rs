use std::{collections::{HashMap}};

use serde::{Serialize, Deserialize};

use crate::{Location, Node};

pub const EPSILON_CHAR: u8 = 0;
const ALPHABET_SIZE: usize = 256;
const INVALID_SYMBOL: usize = usize::MAX;
pub const NOT_ACCEPT:i32 = -1;

/// A runnable deterministic finite automaton.
///
/// This is the actual type that will be executed by the larsty runtime.
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
    pub fn longest_matching<'a>(&self, input:&'a str) -> Result<Vec<Node<(i32, &'a str)>>, String> {
        if input.len() == 0 {
            return Ok(Vec::new());
        }

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
                line = last_accept_line;
                line_char = last_accept_line_char;
                continue;
            }

            // We have lexing error
            return Err(format!("Unexpected character {} at {}:{}", b as char, line, line_char));
        }

        // At end we check if we're at an accept state
        if self.actions[s] == NOT_ACCEPT {
            return Err(String::from("Lexing error"));
        }

        // consume remaining
        let detected = &input[last_consumed_byte as usize..last_accept_byte as usize];

        let location = Location::new(last_accept_line, last_accept_line_char);
        actions.push(Node::new((last_accept_action, detected), location));

        Ok(actions)
    }
}
