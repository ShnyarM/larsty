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
pub enum Type {
    I32
}

#[derive(Debug)]
pub enum RetType {
    Type(Type),
    Void
}

#[derive(Debug)]
pub enum Bop {
    Add,
    Sub,
    Mult
}

#[derive(Debug)]
pub enum Expression {
    Bop(Bop, Box<Node<Expression>>, Box<Node<Expression>>),
    Id(String),
    Val(i32)
}

#[derive(Debug)]
pub enum Statement {
    Decl(String, Node<Expression>),
    Assn(String, Node<Expression>),
    Block(StatementList),
    Expr(Node<Expression>),
    Ret(Option<Node<Expression>>)
}

pub type StatementList = Vec<Node<Statement>>;

#[derive(Debug)]
pub struct FunctionDecl {
    pub name: String,
    pub ret_type: RetType,
    pub code: StatementList
}

impl FunctionDecl {
    pub fn new(name: String, ret_type: RetType, code: StatementList) -> FunctionDecl {
        FunctionDecl { 
            name, 
            ret_type, 
            code 
        }
    }
}

#[derive(Debug)]
pub struct Program {
    pub functions: Vec<Node<FunctionDecl>>
}

impl Program {
    pub fn new() -> Program {
        Program { functions: Vec::new() }
    }

    pub fn add_function(&mut self, func: Node<FunctionDecl>) {
        self.functions.push(func);
    }
}
