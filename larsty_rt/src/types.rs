#[derive(Clone, Debug)]
pub struct Location {
    line: usize,
    char: usize
}

impl Location {
    pub fn new(l: usize, c: usize) -> Location {
        Location {line: l, char: c}
    }
}

pub struct TokenNode {
    token: Token,
    loc: Location
}

/// Used for associativity of operators
#[derive(Clone, Copy)]
pub enum Associativity {
    Right,
    Left,
    NonAssoc
}

