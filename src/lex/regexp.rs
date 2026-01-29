pub enum Regexp{
    Empty,
    String(String),
    Or(Box<Regexp>, Box<Regexp>),
    Concat(Box<Regexp>, Box<Regexp>),
    Rep(Box<Regexp>)
}

impl Regexp {
    pub fn empty() -> Regexp {
        Regexp::Empty
    }

    pub fn string(str: &str) -> Regexp {
        Regexp::String(String::from(str))
    }

    pub fn or(r1: Regexp, r2: Regexp) -> Regexp {
        Regexp::Or(Box::new(r1), Box::new(r2))
    }

    pub fn concat(r1: Regexp, r2: Regexp) -> Regexp {
        Regexp::Concat(Box::new(r1), Box::new(r2))
    }

    pub fn rep(r: Regexp) -> Regexp {
        Regexp::Rep(Box::new(r))
    }
}


