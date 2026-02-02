#[derive(Clone, Debug)] 
pub enum Token { Function,
    Let,
    Id(String),
    Eq,
    If,
    EqEq,
}
