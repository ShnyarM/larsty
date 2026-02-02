pub enum Expression{
    Add(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Val(i32)
}

impl Expression{
    pub fn new_add(e1: Expression, e2: Expression) -> Expression {
        Expression::Add(Box::new(e1), Box::new(e2))
    }

    pub fn new_mul(e1: Expression, e2: Expression) -> Expression {
        Expression::Mul(Box::new(e1), Box::new(e2))
    }

    pub fn new_val(i: i32) -> Expression {
        Expression::Val(i)
    }

    pub fn eval(&self) -> i32 {
        match self{
            Expression::Add(e1, e2) => e1.eval() + e2.eval(),
            Expression::Mul(e1, e2) => e1.eval() * e2.eval(),
            Expression::Val(i) => *i
        }
    }
}
