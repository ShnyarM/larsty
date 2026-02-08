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

/// General struct that holds element as well as location information
#[derive(Debug)]
pub struct Node<T> {
    loc: Location,
    elem: T
}

impl<T> Node<T> {
    pub fn new(elem: T, l: Location) -> Node<T> {
        Node { loc: l, elem }
    }

    pub fn no_loc(elem: T) -> Node<T> {
        Node { loc: Location::new(0, 0), elem }
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

#[derive(Debug)]
pub enum Expression {
    Add(Box<Node<Expression>>, Box<Node<Expression>>),
    Mul(Box<Node<Expression>>, Box<Node<Expression>>),
    Sub(Box<Node<Expression>>, Box<Node<Expression>>),
    Neg(Box<Node<Expression>>),
    Val(i32),
}

impl Expression {
    pub fn eval(&self) -> i32 {
        match self {
            Expression::Add(e1, e2) => e1.get().eval() + e2.get().eval(),
            Expression::Sub(e1, e2) => e1.get().eval() - e2.get().eval(),
            Expression::Mul(e1, e2) => e1.get().eval() * e2.get().eval(),
            Expression::Neg(e) => -e.get().eval(),
            Expression::Val(v) => *v
        }
    }
}

#[derive(Debug)]
pub enum Statement {
    Decl(String, Node<Expression>),
    Assn(String, Node<Expression>),
    Expr(Node<Expression>)
}
