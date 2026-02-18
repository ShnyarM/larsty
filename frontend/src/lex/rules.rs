use super::regexp::Regexp;
use crate::Token;

type TokenGen = dyn Fn(&str) -> Token;

fn rule<F>(r: Regexp, f: F) -> (Regexp, Box<TokenGen>) 
where 
    F: Fn(&str) -> Token + 'static
{
    (r, Box::new(f))
}

fn keyword(s: &str, t: Token) -> (Regexp, Box<TokenGen>) {
    (Regexp::string(s), Box::new(move |_| t.clone()))
}

pub fn get_rules() -> Vec<(Regexp, Box<TokenGen>)> {
    use Token::*;

    let id_chars = Regexp::or(
        Regexp::string("_"), 
        Regexp::or(
            Regexp::letter(), 
            Regexp::digit()
        )
    );

    let int = Regexp::rep(Regexp::digit());
    let identifier = Regexp::concat(Regexp::letter(), Regexp::rep(id_chars));

    vec![
        keyword("function", Function),
        keyword("if",       If),
        keyword("let",      Let),
        keyword("return",   Return),

        keyword("(",       LParen),
        keyword(")",       RParen),
        keyword("{",       LBrack),
        keyword("}",       RBrack),
        keyword(";",       SemiColon),

        keyword("i32",      I32),
        keyword("void",      I32),

        keyword("==",       EqEq),
        keyword("=",        Eq),
        keyword("+",        Plus),
        keyword("*",        Mult),
        keyword("-",        Minus),
        rule(int,           |i| Int(i.trim().parse::<i32>().unwrap())),
        rule(identifier,    |s| Id(String::from(s.trim()))),
    ]
}
