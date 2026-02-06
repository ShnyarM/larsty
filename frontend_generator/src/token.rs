// To add new token:
// 1. Add token enum 
// 2. Increment TOKEN_COUNT
// 3. Add index in token_index()

pub const TOKEN_COUNT: usize = 9;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)] 
pub enum Token { 
    Function,
    Let,
    Id(String),
    Int(i32),
    Eq,
    If,
    EqEq,
    Plus,
    EOF
}

impl Token {
    pub fn index(&self) -> usize {
       match self {
            Token::Function => 0,
            Token::Let => 1,
            Token::Id(_) => 2,
            Token::Int(_) => 3,
            Token::Eq => 4,
            Token::If => 5,
            Token::EqEq => 6,
            Token::Plus => 7,
            Token::EOF => 8
        }
    }
}
