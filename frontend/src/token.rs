// To add new token:
// 1. Add token enum 
// 2. Increment TOKEN_COUNT
// 3. Add index in token_index()

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)] 
pub enum Token { 
    Function,       // function
    Let,            // let
    If,             // if
    Return,         // return

    LParen,         // (
    RParen,         // )
    LBrack,         // {
    RBrack,         // }
    SemiColon,      // ;
             
    Id(String),     // variable/function name
    Int(i32),       // i32 value

    Void,           // void
    I32,            // i32
             
    Eq,             // =
    EqEq,           // ==
    Plus,           // +
    Mult,           // *
    Minus,          // -
             
    EOF             // end of file
}

impl Token {
    pub const COUNT: usize = 3001;

    pub fn index(&self) -> usize {
       match self {
            Token::Function => 0,
            Token::Let => 1,
            Token::If => 2,
            Token::Return => 3,

            Token::LParen => 500,
            Token::RParen => 501,
            Token::LBrack => 502,
            Token::RBrack => 503,
            Token::SemiColon => 504,

            Token::Id(_) => 1000,
            Token::Int(_) => 1001,

            Token::Void => 1500,
            Token::I32 => 1501,

            Token::Eq => 2000,
            Token::EqEq => 2001,
            Token::Plus => 2002,
            Token::Mult => 2003,
            Token::Minus => 2004,

            Token::EOF => 3000 
        }
    }

    pub fn i32(&self) -> i32 {
        match self {
            Token::Int(i) => *i,
            _ => panic!("Token is not an i32")
        }
    }

    pub fn str(self) -> String {
        match self {
            Token::Id(s) => s,
            _ => panic!("Token is not an identifier")
        }
    }
}
