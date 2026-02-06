pub struct Location {
    line: i32,
    char: i32
}

/// General struct that holds element as well as location information
pub struct Node<T> {
    loc: Location,
    elem: T
}

impl<T> Node<T> {
    pub fn new(elem: T, l: Location) -> Node<T> {
        Node { loc: l, elem }
    }

    /// Returns all elements of this Node, destroying it in the process
    pub fn extract(self) -> (T, Location) {
        (self.elem, self.loc)
    }

    pub fn get(&self) -> &T {
        &self.elem
    }
    
    pub fn into_get(self) -> T {
        self.elem
    }

    pub fn loc(&self) -> &Location {
        &self.loc
    }

    pub fn into_loc(self) -> Location {
        self.loc
    }
}

pub enum Expression {
    Add(Box<Node<Expression>>, Box<Node<Expression>>),
    Mul(Box<Node<Expression>>, Box<Node<Expression>>),
    Val(i32)
}

pub enum Statement {
    Decl(String, Node<Expression>),
    Assn(String, Node<Expression>),
    Expr(Node<Expression>)
}
